//! Projectile - the bubble being shot.
//!
//! The projectile travels in a straight line, bouncing off walls,
//! until it hits another bubble or the top of the grid.

use bevy::prelude::*;

use super::{
    bubble::{spawn_bubble, BubbleColor, GameAssets, SNORD_SPRITE_SCALE},
    grid::HexGrid,
    hex::{GridOffset, HexCoord, HEX_SIZE},
    powerups::{PowerUp, UnlockedPowerUps},
    shooter::SHOOTER_Y,
};

use crate::{audio::sound_effect, screens::Screen, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Projectile>();
    app.add_message::<FireProjectile>();
    app.add_message::<BubbleLanded>();
    app.add_message::<BubbleInDangerZone>();

    app.add_systems(
        Update,
        (
            spawn_projectile,
            move_projectile,
            check_wall_collision,
            check_bubble_collision,
        )
            .in_set(PausableSystems)
            .in_set(ProjectileSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Message sent when a bubble lands in the danger zone (triggers game over).
#[derive(Message, Debug, Clone)]
pub struct BubbleInDangerZone;

/// System set for projectile systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectileSystems;

/// Message to fire a projectile.
#[derive(Message, Debug, Clone)]
pub struct FireProjectile {
    pub position: Vec2,
    pub direction: Vec2,
    pub color: BubbleColor,
}

/// Message sent when a bubble lands on the grid.
/// Used to trigger cluster detection.
#[derive(Message, Debug, Clone)]
pub struct BubbleLanded {
    pub coord: HexCoord,
    pub color: BubbleColor,
    #[allow(dead_code)]
    pub entity: Entity,
}

/// Component marking an entity as an active projectile.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Projectile {
    /// Current velocity (direction * speed)
    pub velocity: Vec2,
    /// The bubble color
    pub color: BubbleColor,
}

/// Speed of the projectile in pixels per second.
const PROJECTILE_SPEED: f32 = 600.0;

/// Left wall X position - aligned with left edge of odd row hexes.
/// For q=-6 to 6, odd rows extend to ~242px, walls at ±245 for margin.
pub const LEFT_WALL: f32 = -245.0;

/// Right wall X position - aligned with right edge of odd row hexes.
/// For q=-6 to 6, odd rows extend to ~242px, walls at ±245 for margin.
pub const RIGHT_WALL: f32 = 245.0;

/// Top wall Y position (where projectiles stop).
pub const TOP_WALL: f32 = 280.0;

/// Danger line Y position - bubbles landing below this trigger game over.
pub const DANGER_LINE_Y: f32 = SHOOTER_Y + 80.0;

/// Spawn a projectile when the fire message is received.
fn spawn_projectile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fire_events: MessageReader<FireProjectile>,
    powerups: Res<UnlockedPowerUps>,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
) {
    for event in fire_events.read() {
        // Play launch sound
        let launch_sound = asset_server.load("audio/sound_effects/launch.ogg");
        commands.spawn(sound_effect(launch_sound));
        // Speedy Snord gives 25% faster projectiles
        let speed = if powerups.has(PowerUp::SpeedySnord) {
            PROJECTILE_SPEED * 1.25
        } else {
            PROJECTILE_SPEED
        };
        let velocity = event.direction.normalize() * speed;

        // Check if this color uses a sprite
        let sprite_image = match event.color {
            BubbleColor::Blue => Some(game_assets.derpy_image.clone()),
            BubbleColor::Purple => Some(game_assets.scared_image.clone()),
            BubbleColor::Yellow => Some(game_assets.sad_image.clone()),
            BubbleColor::Red => Some(game_assets.angry_image.clone()),
            BubbleColor::Green => Some(game_assets.happy_image.clone()),
            BubbleColor::Orange => Some(game_assets.enamored_image.clone()),
        };

        if let Some(image) = sprite_image {
            commands.spawn((
                Name::new("Projectile"),
                Projectile {
                    velocity,
                    color: event.color,
                },
                Transform::from_translation(event.position.extend(5.0))
                    .with_scale(Vec3::splat(SNORD_SPRITE_SCALE)),
                Sprite::from_image(image),
                DespawnOnExit(Screen::Gameplay),
            ));
        } else {
            commands.spawn((
                Name::new("Projectile"),
                Projectile {
                    velocity,
                    color: event.color,
                },
                Transform::from_translation(event.position.extend(5.0)),
                Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE, 6))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(event.color.to_color()))),
                DespawnOnExit(Screen::Gameplay),
            ));
        }

        info!("Spawned projectile at {:?} with velocity {:?}", event.position, velocity);
    }
}

/// Move the projectile based on its velocity.
fn move_projectile(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Projectile)>,
) {
    for (mut transform, projectile) in &mut query {
        transform.translation += projectile.velocity.extend(0.0) * time.delta_secs();
    }
}

/// Check for wall collisions and bounce.
fn check_wall_collision(
    mut commands: Commands,
    mut grid: ResMut<HexGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
    mut landed_events: MessageWriter<BubbleLanded>,
    mut danger_events: MessageWriter<BubbleInDangerZone>,
    grid_offset: Res<GridOffset>,
    game_assets: Res<GameAssets>,
) {
    for (entity, mut transform, mut projectile) in &mut query {
        let pos = transform.translation;
        let radius = HEX_SIZE * 0.9;

        // Left wall bounce
        if pos.x - radius < LEFT_WALL {
            transform.translation.x = LEFT_WALL + radius;
            projectile.velocity.x = projectile.velocity.x.abs();
        }

        // Right wall bounce
        if pos.x + radius > RIGHT_WALL {
            transform.translation.x = RIGHT_WALL - radius;
            projectile.velocity.x = -projectile.velocity.x.abs();
        }

        // Top wall - snap to grid
        if pos.y + radius > TOP_WALL {
            let world_pos = pos.truncate();
            if let Some(coord) = grid.closest_empty_cell(world_pos, grid_offset.y) {
                // Check if landing position is in danger zone
                let landing_y = coord.to_pixel_with_offset(HEX_SIZE, grid_offset.y).y;
                if landing_y < DANGER_LINE_Y {
                    info!("Bubble would land in danger zone at y={}, triggering game over", landing_y);
                    danger_events.write(BubbleInDangerZone);
                    commands.entity(entity).despawn();
                } else {
                    let new_entity = land_projectile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &mut grid,
                        entity,
                        coord,
                        projectile.color,
                        grid_offset.y,
                        &game_assets,
                    );
                    landed_events.write(BubbleLanded {
                        coord,
                        color: projectile.color,
                        entity: new_entity,
                    });
                }
            } else {
                // No valid cell found, just despawn
                commands.entity(entity).despawn();
            }
        }

        // Bottom wall - despawn if it somehow goes too low (shouldn't happen)
        if pos.y < SHOOTER_Y - 50.0 {
            warn!("Projectile went below shooter, despawning");
            commands.entity(entity).despawn();
        }
    }
}

/// Check for collision with existing bubbles on the grid.
fn check_bubble_collision(
    mut commands: Commands,
    mut grid: ResMut<HexGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    projectile_query: Query<(Entity, &Transform, &Projectile)>,
    bubble_query: Query<&Transform, Without<Projectile>>,
    mut landed_events: MessageWriter<BubbleLanded>,
    mut danger_events: MessageWriter<BubbleInDangerZone>,
    grid_offset: Res<GridOffset>,
    powerups: Res<UnlockedPowerUps>,
    game_assets: Res<GameAssets>,
) {
    // Sharpshooter reduces collision distance for more precise shots
    let collision_distance = if powerups.has(PowerUp::Sharpshooter) {
        HEX_SIZE * 1.5 // Tighter hitbox
    } else {
        HEX_SIZE * 1.8 // Default: slightly less than 2 radii
    };

    // First pass: find collisions (without borrowing grid mutably)
    let mut collision: Option<(Entity, Vec2, BubbleColor)> = None;

    for (proj_entity, proj_transform, projectile) in &projectile_query {
        let proj_pos = proj_transform.translation.truncate();

        // Check against all grid bubbles
        for (_coord, &bubble_entity) in grid.iter() {
            let Ok(bubble_transform) = bubble_query.get(bubble_entity) else {
                continue;
            };

            let bubble_pos = bubble_transform.translation.truncate();
            let distance = proj_pos.distance(bubble_pos);

            if distance < collision_distance {
                collision = Some((proj_entity, proj_pos, projectile.color));
                break;
            }
        }

        if collision.is_some() {
            break;
        }
    }

    // Second pass: handle the collision (now we can borrow grid mutably)
    if let Some((proj_entity, proj_pos, color)) = collision {
        // Check if projectile position at collision time is in danger zone
        // This must happen BEFORE pathfinding, since pathfinding can find cells above
        if proj_pos.y < DANGER_LINE_Y {
            info!("Projectile collided in danger zone at y={}, triggering game over", proj_pos.y);
            danger_events.write(BubbleInDangerZone);
            commands.entity(proj_entity).despawn();
            return;
        }

        if let Some(snap_coord) = grid.closest_empty_cell(proj_pos, grid_offset.y) {
            let new_entity = land_projectile(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut grid,
                proj_entity,
                snap_coord,
                color,
                grid_offset.y,
                &game_assets,
            );
            landed_events.write(BubbleLanded {
                coord: snap_coord,
                color,
                entity: new_entity,
            });
        } else {
            // No valid cell found, just despawn
            commands.entity(proj_entity).despawn();
        }
    }
}

/// Convert a projectile into a grid bubble.
fn land_projectile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    grid: &mut ResMut<HexGrid>,
    projectile_entity: Entity,
    coord: HexCoord,
    color: BubbleColor,
    grid_origin_y: f32,
    game_assets: &GameAssets,
) -> Entity {
    // Despawn the projectile
    commands.entity(projectile_entity).despawn();

    // Spawn a new bubble at the grid position
    let new_entity = spawn_bubble(commands, meshes, materials, coord, color, grid_origin_y, Some(game_assets));
    grid.insert(coord, new_entity);

    info!("Bubble landed at {} with color {:?}", coord, color);

    new_entity
}
