//! Bubble entities - the main game objects.
//!
//! Bubbles are placed on the hex grid and have different colors.
//! When 3+ of the same color are connected, they pop!

use bevy::prelude::*;
use rand::Rng;

use super::{
    grid::HexGrid,
    hex::{HexCoord, HEX_SIZE},
};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Bubble>();
    app.register_type::<BubbleColor>();

    // Spawn initial bubbles when entering gameplay
    app.add_systems(OnEnter(Screen::Gameplay), spawn_initial_bubbles);

    // Cleanup bubbles when leaving gameplay
    app.add_systems(OnExit(Screen::Gameplay), cleanup_bubbles);
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

    /// Get all possible bubble colors.
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
) {
    info!("Spawning initial bubbles...");

    let bounds = grid.bounds;
    let mut count = 0;

    // Fill the top INITIAL_ROWS rows with random bubbles
    for r in 0..INITIAL_ROWS {
        for q in bounds.min_q..=bounds.max_q {
            let coord = HexCoord::new(q, r);
            let color = BubbleColor::random();

            let entity = spawn_bubble(&mut commands, &mut meshes, &mut materials, coord, color);
            grid.insert(coord, entity);
            count += 1;
        }
    }

    info!("Spawned {} initial bubbles", count);
}

/// Spawn a single bubble at the given hex coordinate with the given color.
pub fn spawn_bubble(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    coord: HexCoord,
    color: BubbleColor,
) -> Entity {
    let world_pos = coord.to_pixel(HEX_SIZE);

    // Create a hexagon mesh for the bubble
    // RegularPolygon::new(circumradius, sides) - circumradius = HEX_SIZE
    // Rotate 30° (FRAC_PI_6) for pointy-top orientation to match grid
    commands
        .spawn((
            Name::new(format!("Bubble {:?} at {}", color, coord)),
            Bubble { color, coord },
            color,
            // Transform with rotation for pointy-top hexagon
            Transform::from_translation(world_pos.extend(0.0))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_6)),
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
