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
    pub fn to_color(&self) -> Color {
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
