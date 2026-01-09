//! Bubble entities - the main game objects.
//!
//! Bubbles are placed on the hex grid and have different colors.
//! When 3+ of the same color are connected, they pop!

use bevy::prelude::*;
use rand::Rng;

use super::{
    grid::HexGrid,
    hex::{GridOffset, HEX_SIZE, HexCoord},
};
use crate::screens::Screen;

/// Holds game asset handles for bubble rendering.
#[derive(Resource)]
pub struct GameAssets {
    pub derpy_image: Handle<Image>,
    pub scared_image: Handle<Image>,
    pub sad_image: Handle<Image>,
    pub angry_image: Handle<Image>,
    pub happy_image: Handle<Image>,
    pub enamored_image: Handle<Image>,
    pub shooter_image: Handle<Image>,
    pub guide_line_image: Handle<Image>,
    pub doodle_images: Vec<Handle<Image>>,
}

/// Scale factor for snord sprites (64px -> ~40px to match HEX_SIZE diameter).
pub const SNORD_SPRITE_SCALE: f32 = 0.625;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Bubble>();
    app.register_type::<BubbleColor>();

    // Load game assets before spawning bubbles
    app.add_systems(
        OnEnter(Screen::Gameplay),
        load_game_assets.before(spawn_initial_bubbles),
    );

    // Spawn initial bubbles when entering gameplay
    app.add_systems(OnEnter(Screen::Gameplay), spawn_initial_bubbles);

    // Spawn background doodles after assets are loaded
    app.add_systems(
        OnEnter(Screen::Gameplay),
        spawn_background_doodles.after(load_game_assets),
    );

    // Cleanup bubbles when leaving gameplay
    app.add_systems(OnExit(Screen::Gameplay), cleanup_bubbles);
}

/// Load game assets - must run before any systems that use GameAssets.
pub fn load_game_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        derpy_image: asset_server.load("images/derpy.png"),
        scared_image: asset_server.load("images/scared.png"),
        sad_image: asset_server.load("images/sad.png"),
        angry_image: asset_server.load("images/angry.png"),
        happy_image: asset_server.load("images/happy.png"),
        enamored_image: asset_server.load("images/enamored.png"),
        shooter_image: asset_server.load("images/shooter.png"),
        guide_line_image: asset_server.load("images/guide_line.png"),
        doodle_images: vec![
            asset_server.load("images/doodle_1.png"),
            asset_server.load("images/doodle_2.png"),
            asset_server.load("images/doodle_3.png"),
            asset_server.load("images/doodle_4.png"),
            asset_server.load("images/doodle_5.png"),
        ],
    });
}

/// The different bubble colors.
/// Using 6 colors like classic Snood.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Reflect, Default)]
#[reflect(Component)]
pub enum BubbleColor {
    #[default]
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
    Orange,
}

impl BubbleColor {
    /// Get the actual color for rendering.
    pub fn to_color(self) -> Color {
        match self {
            BubbleColor::Red => Color::srgb(0.9, 0.2, 0.2),
            BubbleColor::Blue => Color::srgb(0.2, 0.4, 0.9),
            BubbleColor::Green => Color::srgb(0.2, 0.8, 0.3),
            BubbleColor::Yellow => Color::srgb(0.95, 0.85, 0.2),
            BubbleColor::Purple => Color::srgb(0.7, 0.3, 0.8),
            BubbleColor::Orange => Color::srgb(0.95, 0.5, 0.1),
        }
    }

    /// Get a random bubble color.
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..6) {
            0 => BubbleColor::Red,
            1 => BubbleColor::Blue,
            2 => BubbleColor::Green,
            3 => BubbleColor::Yellow,
            4 => BubbleColor::Purple,
            _ => BubbleColor::Orange,
        }
    }

    /// Get a random color weighted toward colors that exist on the grid.
    /// With Lucky Snord, there's a 70% chance to pick from existing grid colors.
    pub fn random_weighted(grid_colors: &[BubbleColor]) -> Self {
        if grid_colors.is_empty() {
            return Self::random();
        }

        let mut rng = rand::rng();
        // 70% chance to pick from existing grid colors
        if rng.random_bool(0.7) {
            // Pick a random color from the grid
            let idx = rng.random_range(0..grid_colors.len());
            grid_colors[idx]
        } else {
            Self::random()
        }
    }

    /// Get all possible bubble colors.
    #[allow(dead_code)]
    pub const ALL: [BubbleColor; 6] = [
        BubbleColor::Red,
        BubbleColor::Blue,
        BubbleColor::Green,
        BubbleColor::Yellow,
        BubbleColor::Purple,
        BubbleColor::Orange,
    ];
}

/// Marker component for bubble entities.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Bubble {
    /// The bubble's color (also stored as a separate component for easy querying)
    pub color: BubbleColor,
    /// The hex coordinate where this bubble is placed
    pub coord: HexCoord,
}

/// Number of rows to fill at the start of the game.
const INITIAL_ROWS: i32 = 5;

/// Spawn the initial bubbles at the top of the grid.
fn spawn_initial_bubbles(
    mut commands: Commands,
    mut grid: ResMut<HexGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    grid_offset: Res<GridOffset>,
    game_assets: Res<GameAssets>,
) {
    info!("Spawning initial bubbles...");

    let bounds = grid.bounds;
    let mut count = 0;

    // Fill the top INITIAL_ROWS rows with random bubbles
    for r in 0..INITIAL_ROWS {
        for q in bounds.min_q..=bounds.max_q {
            let coord = HexCoord::new(q, r);
            let color = BubbleColor::random();

            let entity = spawn_bubble(
                &mut commands,
                &mut meshes,
                &mut materials,
                coord,
                color,
                grid_offset.y,
                Some(&game_assets),
            );
            grid.insert(coord, entity);
            count += 1;
        }
    }

    info!("Spawned {} initial bubbles", count);
}

/// Spawn a single bubble at the given hex coordinate with the given color.
/// If game_assets is provided and color is Blue, uses derpy sprite instead of mesh.
pub fn spawn_bubble(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    coord: HexCoord,
    color: BubbleColor,
    grid_origin_y: f32,
    game_assets: Option<&GameAssets>,
) -> Entity {
    let world_pos = coord.to_pixel_with_offset(HEX_SIZE, grid_origin_y);

    // For certain colors, use sprite images instead of colored meshes
    if let Some(assets) = game_assets {
        let sprite_image = match color {
            BubbleColor::Blue => Some(assets.derpy_image.clone()),
            BubbleColor::Purple => Some(assets.scared_image.clone()),
            BubbleColor::Yellow => Some(assets.sad_image.clone()),
            BubbleColor::Red => Some(assets.angry_image.clone()),
            BubbleColor::Green => Some(assets.happy_image.clone()),
            BubbleColor::Orange => Some(assets.enamored_image.clone()),
        };

        if let Some(image) = sprite_image {
            return commands
                .spawn((
                    Name::new(format!("Bubble {:?} at {}", color, coord)),
                    Bubble { color, coord },
                    color,
                    Transform::from_translation(world_pos.extend(0.0))
                        .with_scale(Vec3::splat(SNORD_SPRITE_SCALE)),
                    Sprite::from_image(image),
                    DespawnOnExit(Screen::Gameplay),
                ))
                .id();
        }
    }

    // Default: Create a hexagon mesh for the bubble
    // RegularPolygon::new(circumradius, sides) - circumradius = HEX_SIZE
    commands
        .spawn((
            Name::new(format!("Bubble {:?} at {}", color, coord)),
            Bubble { color, coord },
            color,
            Transform::from_translation(world_pos.extend(0.0)),
            // Hexagon mesh
            Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE, 6))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color.to_color()))),
            // Mark for cleanup when leaving gameplay
            DespawnOnExit(Screen::Gameplay),
        ))
        .id()
}

/// Remove all bubble entities when leaving gameplay.
fn cleanup_bubbles(mut grid: ResMut<HexGrid>) {
    grid.clear();
    info!("Cleared bubble grid");
}

/// Spawn decorative doodles in the background on left/right sides of the game area.
fn spawn_background_doodles(mut commands: Commands, game_assets: Res<GameAssets>) {
    let mut rng = rand::rng();

    // Game bounds are -245 to +245, window is -400 to +400
    // Left margin: -400 to -260 (keeping buffer from game)
    // Right margin: +260 to +400
    const DOODLE_SIZE: f32 = 70.0; // Approximate size of doodle at scale 1.0
    const SCALE: f32 = 0.45; // Smaller scale to fit more
    const CELL_SIZE: f32 = DOODLE_SIZE * SCALE; // Grid cell size (~31px)
    const JITTER: f32 = 12.0; // Random offset to break up grid pattern

    // Define margin boundaries (stay away from game area)
    const LEFT_MIN: f32 = -395.0;
    const LEFT_MAX: f32 = -260.0;
    const RIGHT_MIN: f32 = 260.0;
    const RIGHT_MAX: f32 = 395.0;
    const Y_MIN: f32 = -290.0;
    const Y_MAX: f32 = 290.0;

    let margin_width = LEFT_MAX - LEFT_MIN; // ~130px
    let margin_height = Y_MAX - Y_MIN; // ~580px

    // Calculate grid dimensions
    let cols = (margin_width / CELL_SIZE).floor() as i32;
    let rows = (margin_height / CELL_SIZE).floor() as i32;

    let mut count = 0;

    // Spawn doodles on both sides using grid placement
    for (min_x, _max_x) in [(LEFT_MIN, LEFT_MAX), (RIGHT_MIN, RIGHT_MAX)] {
        for col in 0..cols {
            for row in 0..rows {
                // Grid position with small jitter
                let base_x = min_x + (col as f32 + 0.5) * CELL_SIZE;
                let base_y = Y_MIN + (row as f32 + 0.5) * CELL_SIZE;

                let x = base_x + rng.random_range(-JITTER..JITTER);
                let y = base_y + rng.random_range(-JITTER..JITTER);

                // Pick a random doodle image
                let doodle_idx = rng.random_range(0..game_assets.doodle_images.len());
                let image = game_assets.doodle_images[doodle_idx].clone();

                // Random rotation (full 360 degrees)
                let rotation = rng.random_range(0.0..std::f32::consts::TAU);

                // Slight scale variation
                let scale = SCALE + rng.random_range(-0.05..0.05);

                commands.spawn((
                    Name::new(format!("Background Doodle {}", doodle_idx + 1)),
                    Transform::from_translation(Vec3::new(x, y, -1.0))
                        .with_rotation(Quat::from_rotation_z(rotation))
                        .with_scale(Vec3::splat(scale)),
                    Sprite::from_image(image),
                    DespawnOnExit(Screen::Gameplay),
                ));
                count += 1;
            }
        }
    }

    info!(
        "Spawned {} background doodles ({}x{} grid per side)",
        count, cols, rows
    );
}
