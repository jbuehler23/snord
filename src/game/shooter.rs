//! The shooter/launcher at the bottom of the screen.
//!
//! The player aims with the mouse and fires bubbles upward.
//! The shooter always has a "loaded" bubble ready to fire and
//! a "next" bubble preview.

use bevy::{prelude::*, window::PrimaryWindow};

use super::{
    bubble::{Bubble, BubbleColor, GameAssets, SNORD_SPRITE_SCALE, load_game_assets},
    grid::HexGrid,
    hex::HEX_SIZE,
    powerups::{PowerUp, UnlockedPowerUps},
    projectile::{FireProjectile, LEFT_WALL, Projectile, RIGHT_WALL},
    state::{GameLevel, TriggerDescent},
};
use crate::{PausableSystems, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Shooter>();
    app.register_type::<ShooterState>();
    app.register_type::<AimDirection>();
    app.register_type::<NextBubble>();

    // Spawn shooter when entering gameplay (after assets are loaded)
    app.add_systems(
        OnEnter(Screen::Gameplay),
        spawn_shooter.after(load_game_assets),
    );

    // Update systems that run while playing
    app.add_systems(
        Update,
        (
            update_aim_direction,
            update_shooter_arrow_rotation,
            draw_aim_line,
            handle_fire_input,
            reload_shooter,
            update_fortune_snord_visibility,
        )
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// The Y position of the shooter (bottom of play area).
pub const SHOOTER_Y: f32 = -250.0;

/// Maximum angle from vertical (in radians) - prevents shooting too horizontally.
const MAX_AIM_ANGLE: f32 = 1.3; // About 75 degrees

/// Base length of the aim line in pixels (doubled with Eagle Eye power-up).
const AIM_LINE_LENGTH: f32 = 150.0;

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

/// The second next bubble color (Fortune Snord preview).
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct SecondNextBubble(pub BubbleColor);

/// The third next bubble color (Fortune Snord preview).
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ThirdNextBubble(pub BubbleColor);

/// Marker for the loaded bubble visual entity.
#[derive(Component)]
struct LoadedBubbleVisual;

/// Marker for the next bubble visual entity.
#[derive(Component)]
struct NextBubbleVisual;

/// Marker for the second next bubble visual entity.
#[derive(Component)]
struct SecondNextBubbleVisual;

/// Marker for the third next bubble visual entity.
#[derive(Component)]
struct ThirdNextBubbleVisual;

/// Marker for the shooter arrow visual entity.
#[derive(Component)]
struct ShooterArrowVisual;

/// Spawn the shooter at the bottom of the screen.
fn spawn_shooter(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_assets: Res<GameAssets>,
) {
    info!("Spawning shooter at y={}", SHOOTER_Y);

    let loaded_color = BubbleColor::random();
    let next_color = BubbleColor::random();
    let second_next_color = BubbleColor::random();
    let third_next_color = BubbleColor::random();

    // Main shooter entity
    let shooter_entity = commands
        .spawn((
            Name::new("Shooter"),
            Shooter,
            ShooterState::Ready,
            AimDirection::default(),
            LoadedBubble(loaded_color),
            NextBubble(next_color),
            SecondNextBubble(second_next_color),
            ThirdNextBubble(third_next_color),
            Transform::from_xyz(0.0, SHOOTER_Y, 1.0),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    // Spawn the shooter arrow visual (follows aim direction)
    // Anchor at bottom so rotation pivot matches guide line origin
    let arrow = commands
        .spawn((
            Name::new("Shooter Arrow"),
            ShooterArrowVisual,
            Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            Sprite {
                image: game_assets.shooter_image.clone(),
                // Stretch to make it longer (64x64 -> 64x128)
                custom_size: Some(Vec2::new(64.0, 128.0)),
                ..default()
            },
            bevy::sprite::Anchor::BOTTOM_CENTER,
        ))
        .id();
    commands.entity(shooter_entity).add_child(arrow);

    // Spawn preview bubble visuals as children
    spawn_bubble_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        &game_assets,
        shooter_entity,
        loaded_color,
        Vec3::ZERO,
        1.0,
        LoadedBubbleVisual,
        Visibility::Inherited,
    );

    // Base/platform visual
    let base = commands
        .spawn((
            Name::new("Shooter Base"),
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.35),
                custom_size: Some(Vec2::new(HEX_SIZE * 3.0, HEX_SIZE * 0.5)),
                ..default()
            },
            Transform::from_xyz(0.0, -HEX_SIZE * 1.2, -0.1),
        ))
        .id();
    commands.entity(shooter_entity).add_child(base);

    spawn_bubble_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        &game_assets,
        shooter_entity,
        next_color,
        Vec3::new(HEX_SIZE * 3.0, 0.0, 0.0),
        0.6,
        NextBubbleVisual,
        Visibility::Inherited,
    );

    spawn_bubble_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        &game_assets,
        shooter_entity,
        second_next_color,
        Vec3::new(HEX_SIZE * 4.5, 0.0, 0.0),
        0.5,
        SecondNextBubbleVisual,
        Visibility::Hidden,
    );

    spawn_bubble_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        &game_assets,
        shooter_entity,
        third_next_color,
        Vec3::new(HEX_SIZE * 5.8, 0.0, 0.0),
        0.4,
        ThirdNextBubbleVisual,
        Visibility::Hidden,
    );

    info!(
        "Shooter spawned with {:?} loaded, {:?} next",
        loaded_color, next_color
    );
}

/// Spawn a bubble visual (sprite for blue, mesh for others) as a child of the given parent.
fn spawn_bubble_visual<M: Component>(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    game_assets: &GameAssets,
    parent: Entity,
    color: BubbleColor,
    position: Vec3,
    scale: f32,
    marker: M,
    visibility: Visibility,
) {
    // Check if this color uses a sprite
    let sprite_image = match color {
        BubbleColor::Blue => Some(game_assets.derpy_image.clone()),
        BubbleColor::Purple => Some(game_assets.scared_image.clone()),
        BubbleColor::Yellow => Some(game_assets.sad_image.clone()),
        BubbleColor::Red => Some(game_assets.angry_image.clone()),
        BubbleColor::Green => Some(game_assets.happy_image.clone()),
        BubbleColor::Orange => Some(game_assets.enamored_image.clone()),
    };

    if let Some(image) = sprite_image {
        let child = commands
            .spawn((
                Name::new("Bubble Visual (Sprite)"),
                marker,
                Transform::from_translation(position)
                    .with_scale(Vec3::splat(SNORD_SPRITE_SCALE * scale)),
                Sprite::from_image(image),
                visibility,
            ))
            .id();
        commands.entity(parent).add_child(child);
    } else {
        // Use colored hexagon mesh for other colors
        let child = commands
            .spawn((
                Name::new("Bubble Visual (Mesh)"),
                marker,
                Transform::from_translation(position),
                Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE * scale, 6))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color.to_color()))),
                visibility,
            ))
            .id();
        commands.entity(parent).add_child(child);
    }
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

/// Update the shooter arrow rotation to match aim direction.
fn update_shooter_arrow_rotation(
    shooter_query: Query<&AimDirection, With<Shooter>>,
    mut arrow_query: Query<&mut Transform, With<ShooterArrowVisual>>,
) {
    let Ok(aim) = shooter_query.single() else {
        return;
    };
    let Ok(mut arrow_transform) = arrow_query.single_mut() else {
        return;
    };

    // Calculate rotation angle from aim direction
    // atan2(x, y) gives angle from vertical (Y-axis)
    let angle = -aim.0.x.atan2(aim.0.y);
    arrow_transform.rotation = Quat::from_rotation_z(angle);
}

/// Draw the aim line using gizmos.
fn draw_aim_line(
    mut gizmos: Gizmos,
    shooter_query: Query<(&Transform, &AimDirection, &ShooterState), With<Shooter>>,
    powerups: Res<UnlockedPowerUps>,
) {
    let Ok((transform, aim, state)) = shooter_query.single() else {
        return;
    };

    // Don't draw aim line while reloading
    if *state == ShooterState::Reloading {
        return;
    }

    let start = transform.translation.truncate();

    // Eagle Eye doubles the aim line length
    let aim_length = if powerups.has(PowerUp::EagleEye) {
        AIM_LINE_LENGTH * 2.0
    } else {
        AIM_LINE_LENGTH
    };

    // Bouncy Snord: Show bounce trajectory
    if powerups.has(PowerUp::BouncySnord) {
        draw_bouncy_aim_line(&mut gizmos, start, aim.0, aim_length);
    } else {
        draw_simple_aim_line(&mut gizmos, start, aim.0, aim_length);
    }
}

/// Draw a simple dotted aim line (dark, behind shooter sprite).
fn draw_simple_aim_line(gizmos: &mut Gizmos, start: Vec2, direction: Vec2, length: f32) {
    let segments = 15;
    let segment_length = length / segments as f32;

    for i in 0..segments {
        if i % 2 == 0 {
            let seg_start = start + direction * (i as f32 * segment_length);
            let seg_end = start + direction * ((i as f32 + 0.7) * segment_length);
            gizmos.line_2d(seg_start, seg_end, Color::srgba(0.1, 0.1, 0.1, 0.6));
        }
    }
}

/// Draw aim line with bounce trajectory (Bouncy Snord power-up).
fn draw_bouncy_aim_line(gizmos: &mut Gizmos, start: Vec2, direction: Vec2, total_length: f32) {
    let radius = HEX_SIZE * 0.9; // Projectile collision radius
    let mut current_pos = start;
    let mut current_dir = direction;
    let mut remaining_length = total_length;
    let mut bounces = 0;
    const MAX_BOUNCES: i32 = 3;

    while remaining_length > 0.0 && bounces <= MAX_BOUNCES {
        // Calculate how far until we hit a wall
        let dist_to_left = if current_dir.x < 0.0 {
            ((LEFT_WALL + radius) - current_pos.x) / current_dir.x
        } else {
            f32::MAX
        };
        let dist_to_right = if current_dir.x > 0.0 {
            ((RIGHT_WALL - radius) - current_pos.x) / current_dir.x
        } else {
            f32::MAX
        };

        let dist_to_wall = dist_to_left.min(dist_to_right);
        let segment_length = remaining_length.min(dist_to_wall);

        // Draw this segment as dotted line
        let end_pos = current_pos + current_dir * segment_length;
        let segment_count = (segment_length / 10.0).max(1.0) as i32;
        let seg_len = segment_length / segment_count as f32;

        for i in 0..segment_count {
            if i % 2 == 0 {
                let seg_start = current_pos + current_dir * (i as f32 * seg_len);
                let seg_end = current_pos + current_dir * ((i as f32 + 0.7) * seg_len);
                let alpha = if bounces == 0 { 0.6 } else { 0.4 };
                gizmos.line_2d(seg_start, seg_end, Color::srgba(0.1, 0.1, 0.1, alpha));
            }
        }

        remaining_length -= segment_length;
        current_pos = end_pos;

        // If we hit a wall, bounce
        if remaining_length > 0.0 && dist_to_wall <= segment_length + 0.1 {
            current_dir.x = -current_dir.x; // Reflect horizontally
            bounces += 1;
        }
    }
}

/// Handle fire input (mouse click or spacebar).
fn handle_fire_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut shooter_query: Query<
        (&Transform, &AimDirection, &mut ShooterState, &LoadedBubble),
        With<Shooter>,
    >,
    projectile_query: Query<&Projectile>,
    mut fire_events: MessageWriter<FireProjectile>,
    mut level: ResMut<GameLevel>,
) {
    // Check for fire input
    let fire_pressed =
        mouse_input.just_pressed(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::Space);

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
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut shooter_query: Query<
        (
            Entity,
            &mut ShooterState,
            &mut LoadedBubble,
            &mut NextBubble,
            &mut SecondNextBubble,
            &mut ThirdNextBubble,
        ),
        With<Shooter>,
    >,
    loaded_visual_query: Query<Entity, With<LoadedBubbleVisual>>,
    next_visual_query: Query<Entity, With<NextBubbleVisual>>,
    second_visual_query: Query<Entity, With<SecondNextBubbleVisual>>,
    third_visual_query: Query<Entity, With<ThirdNextBubbleVisual>>,
    projectile_query: Query<&Projectile>,
    level: Res<GameLevel>,
    mut descent_events: MessageWriter<TriggerDescent>,
    powerups: Res<UnlockedPowerUps>,
    grid: Res<HexGrid>,
    bubble_query: Query<&Bubble>,
    game_assets: Res<GameAssets>,
) {
    let Ok((shooter_entity, mut state, mut loaded, mut next, mut second_next, mut third_next)) =
        shooter_query.single_mut()
    else {
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

    // Cycle through all preview bubbles: loaded <- next <- second <- third <- new
    loaded.0 = next.0;
    next.0 = second_next.0;
    second_next.0 = third_next.0;

    // Generate new third preview color
    // Lucky Snord: Weight color selection toward colors on the grid
    if powerups.has(PowerUp::LuckySnord) {
        let grid_colors: Vec<BubbleColor> = grid
            .iter()
            .filter_map(|(_, &entity)| bubble_query.get(entity).ok())
            .map(|b| b.color)
            .collect();
        third_next.0 = BubbleColor::random_weighted(&grid_colors);
    } else {
        third_next.0 = BubbleColor::random();
    }

    // Despawn old visuals and spawn new ones with correct rendering
    if let Ok(entity) = loaded_visual_query.single() {
        commands.entity(entity).despawn();
    }
    spawn_bubble_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        &game_assets,
        shooter_entity,
        loaded.0,
        Vec3::ZERO,
        1.0,
        LoadedBubbleVisual,
        Visibility::Inherited,
    );

    if let Ok(entity) = next_visual_query.single() {
        commands.entity(entity).despawn();
    }
    spawn_bubble_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        &game_assets,
        shooter_entity,
        next.0,
        Vec3::new(HEX_SIZE * 3.0, 0.0, 0.0),
        0.6,
        NextBubbleVisual,
        Visibility::Inherited,
    );

    if let Ok(entity) = second_visual_query.single() {
        commands.entity(entity).despawn();
    }
    spawn_bubble_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        &game_assets,
        shooter_entity,
        second_next.0,
        Vec3::new(HEX_SIZE * 4.5, 0.0, 0.0),
        0.5,
        SecondNextBubbleVisual,
        Visibility::Hidden,
    );

    if let Ok(entity) = third_visual_query.single() {
        commands.entity(entity).despawn();
    }
    spawn_bubble_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        &game_assets,
        shooter_entity,
        third_next.0,
        Vec3::new(HEX_SIZE * 5.8, 0.0, 0.0),
        0.4,
        ThirdNextBubbleVisual,
        Visibility::Hidden,
    );

    *state = ShooterState::Ready;
    info!("Reloaded with {:?}, next is {:?}", loaded.0, next.0);

    // Check if it's time for descent
    // Procrastisnord: +2 extra shots before descent
    let shots_threshold = if powerups.has(PowerUp::Procrastisnord) {
        level.shots_until_descent + 2
    } else {
        level.shots_until_descent
    };

    if level.shots_this_round >= shots_threshold {
        info!(
            "Triggering descent! ({} shots reached, threshold was {})",
            level.shots_this_round, shots_threshold
        );
        descent_events.write(TriggerDescent);
    }
}

/// Update visibility of extra preview bubbles based on Fortune Snord power-up.
fn update_fortune_snord_visibility(
    mut second_query: Query<&mut Visibility, With<SecondNextBubbleVisual>>,
    mut third_query: Query<
        &mut Visibility,
        (With<ThirdNextBubbleVisual>, Without<SecondNextBubbleVisual>),
    >,
    powerups: Res<UnlockedPowerUps>,
) {
    let has_fortune = powerups.has(PowerUp::FortuneSnord);

    // Update visibility of extra preview bubbles
    if let Ok(mut vis) = second_query.single_mut() {
        *vis = if has_fortune {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = third_query.single_mut() {
        *vis = if has_fortune {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
