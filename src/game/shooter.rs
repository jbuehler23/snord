//! The shooter/launcher at the bottom of the screen.
//!
//! The player aims with the mouse and fires bubbles upward.
//! The shooter always has a "loaded" bubble ready to fire and
//! a "next" bubble preview.

use bevy::{prelude::*, window::PrimaryWindow};

use super::{
    bubble::BubbleColor,
    hex::HEX_SIZE,
    projectile::{FireProjectile, Projectile},
    state::{GameLevel, TriggerDescent},
};
use crate::{screens::Screen, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Shooter>();
    app.register_type::<ShooterState>();
    app.register_type::<AimDirection>();
    app.register_type::<NextBubble>();

    // Spawn shooter when entering gameplay
    app.add_systems(OnEnter(Screen::Gameplay), spawn_shooter);

    // Update systems that run while playing
    app.add_systems(
        Update,
        (
            update_aim_direction,
            draw_aim_line,
            handle_fire_input,
            reload_shooter,
        )
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// The Y position of the shooter (bottom of play area).
pub const SHOOTER_Y: f32 = -250.0;

/// Maximum angle from vertical (in radians) - prevents shooting too horizontally.
const MAX_AIM_ANGLE: f32 = 1.3; // About 75 degrees

/// Length of the aim line in pixels.
const AIM_LINE_LENGTH: f32 = 300.0;

/// Marker component for the shooter entity.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Shooter;

/// The current state of the shooter.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
#[reflect(Component)]
pub enum ShooterState {
    /// Ready to fire
    #[default]
    Ready,
    /// Waiting for projectile to land before reloading
    Reloading,
}

/// The current aim direction (normalized vector pointing from shooter).
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AimDirection(pub Vec2);

impl Default for AimDirection {
    fn default() -> Self {
        Self(Vec2::Y) // Start aiming straight up
    }
}

/// The currently loaded bubble color.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct LoadedBubble(pub BubbleColor);

/// The next bubble color (preview).
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct NextBubble(pub BubbleColor);

/// Spawn the shooter at the bottom of the screen.
fn spawn_shooter(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Spawning shooter at y={}", SHOOTER_Y);

    let loaded_color = BubbleColor::random();
    let next_color = BubbleColor::random();

    // Main shooter entity
    commands
        .spawn((
            Name::new("Shooter"),
            Shooter,
            ShooterState::Ready,
            AimDirection::default(),
            LoadedBubble(loaded_color),
            NextBubble(next_color),
            Transform::from_xyz(0.0, SHOOTER_Y, 1.0),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .with_children(|parent| {
            // The loaded bubble visual (hexagon at shooter position)
            parent.spawn((
                Name::new("Loaded Bubble Display"),
                Transform::default(),
                Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE, 6))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(loaded_color.to_color()))),
            ));

            // Base/platform visual
            parent.spawn((
                Name::new("Shooter Base"),
                Sprite {
                    color: Color::srgb(0.3, 0.3, 0.35),
                    custom_size: Some(Vec2::new(HEX_SIZE * 3.0, HEX_SIZE * 0.5)),
                    ..default()
                },
                Transform::from_xyz(0.0, -HEX_SIZE * 1.2, -0.1),
            ));

            // Next bubble preview (smaller hexagon to the side)
            parent.spawn((
                Name::new("Next Bubble Preview"),
                Transform::from_xyz(HEX_SIZE * 3.0, 0.0, 0.0),
                Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE * 0.6, 6))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(next_color.to_color()))),
            ));
        });

    info!("Shooter spawned with {:?} loaded, {:?} next", loaded_color, next_color);
}

/// Update the aim direction based on mouse position.
fn update_aim_direction(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut shooter_query: Query<(&Transform, &mut AimDirection), With<Shooter>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok((shooter_transform, mut aim)) = shooter_query.single_mut() else {
        return;
    };

    // Get cursor position in world coordinates
    let Some(cursor_pos) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(camera_transform, p).ok())
    else {
        return;
    };

    // Calculate direction from shooter to cursor
    let shooter_pos = shooter_transform.translation.truncate();
    let mut direction = (cursor_pos - shooter_pos).normalize_or_zero();

    // Ensure we're aiming upward (not down)
    if direction.y < 0.1 {
        direction.y = 0.1;
        direction = direction.normalize();
    }

    // Clamp angle to prevent too-horizontal shots
    let angle = direction.x.atan2(direction.y);
    let clamped_angle = angle.clamp(-MAX_AIM_ANGLE, MAX_AIM_ANGLE);

    aim.0 = Vec2::new(clamped_angle.sin(), clamped_angle.cos());
}

/// Draw the aim line using gizmos.
fn draw_aim_line(
    mut gizmos: Gizmos,
    shooter_query: Query<(&Transform, &AimDirection, &ShooterState), With<Shooter>>,
) {
    let Ok((transform, aim, state)) = shooter_query.single() else {
        return;
    };

    // Don't draw aim line while reloading
    if *state == ShooterState::Reloading {
        return;
    }

    let start = transform.translation.truncate();

    // Draw a dotted/dashed aim line
    let segments = 15;
    let segment_length = AIM_LINE_LENGTH / segments as f32;

    for i in 0..segments {
        if i % 2 == 0 {
            let seg_start = start + aim.0 * (i as f32 * segment_length);
            let seg_end = start + aim.0 * ((i as f32 + 0.7) * segment_length);
            gizmos.line_2d(seg_start, seg_end, Color::srgba(1.0, 1.0, 1.0, 0.5));
        }
    }
}

/// Handle fire input (mouse click or spacebar).
fn handle_fire_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut shooter_query: Query<
        (
            &Transform,
            &AimDirection,
            &mut ShooterState,
            &LoadedBubble,
        ),
        With<Shooter>,
    >,
    projectile_query: Query<&Projectile>,
    mut fire_events: MessageWriter<FireProjectile>,
    mut level: ResMut<GameLevel>,
) {
    // Check for fire input
    let fire_pressed = mouse_input.just_pressed(MouseButton::Left)
        || keyboard_input.just_pressed(KeyCode::Space);

    if !fire_pressed {
        return;
    }

    let Ok((transform, aim, mut state, loaded)) = shooter_query.single_mut() else {
        return;
    };

    // Can't fire if not ready or if there's already a projectile in flight
    if *state != ShooterState::Ready {
        return;
    }

    // Check if there's already a projectile
    if !projectile_query.is_empty() {
        return;
    }

    // Fire!
    let spawn_pos = transform.translation.truncate();

    fire_events.write(FireProjectile {
        position: spawn_pos,
        direction: aim.0,
        color: loaded.0,
    });

    *state = ShooterState::Reloading;

    // Track shots for descent system
    level.shots_this_round += 1;
    info!(
        "Fired {:?} bubble in direction {:?} (shot {}/{})",
        loaded.0, aim.0, level.shots_this_round, level.shots_until_descent
    );
}

/// Reload the shooter after the projectile lands.
fn reload_shooter(
    mut shooter_query: Query<
        (
            &mut ShooterState,
            &mut LoadedBubble,
            &mut NextBubble,
            &Children,
        ),
        With<Shooter>,
    >,
    mut material_query: Query<&mut MeshMaterial2d<ColorMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    projectile_query: Query<&Projectile>,
    level: Res<GameLevel>,
    mut descent_events: MessageWriter<TriggerDescent>,
) {
    let Ok((mut state, mut loaded, mut next, children)) = shooter_query.single_mut() else {
        return;
    };

    // Only reload when in reloading state and projectile has landed
    if *state != ShooterState::Reloading {
        return;
    }

    // Wait for projectile to be gone
    if !projectile_query.is_empty() {
        return;
    }

    // Move next to loaded, generate new next
    loaded.0 = next.0;
    next.0 = BubbleColor::random();

    // Update the visual materials
    // Children: [0] = loaded bubble display, [1] = base, [2] = next preview
    if children.len() >= 3 {
        if let Ok(mut mat_handle) = material_query.get_mut(children[0]) {
            mat_handle.0 = materials.add(ColorMaterial::from_color(loaded.0.to_color()));
        }
        if let Ok(mut mat_handle) = material_query.get_mut(children[2]) {
            mat_handle.0 = materials.add(ColorMaterial::from_color(next.0.to_color()));
        }
    }

    *state = ShooterState::Ready;
    info!("Reloaded with {:?}, next is {:?}", loaded.0, next.0);

    // Check if it's time for descent
    if level.shots_this_round >= level.shots_until_descent {
        info!(
            "Triggering descent! ({} shots reached)",
            level.shots_this_round
        );
        descent_events.write(TriggerDescent);
    }
}
