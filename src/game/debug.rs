//! Debug visualization for the hexagonal grid.
//!
//! Toggle with the 'D' key during gameplay.
//! Shows:
//! - Hex cell outlines for all valid positions
//! - Occupied cells highlighted
//! - Coordinate labels (when zoomed in)

use bevy::{color::palettes::css, input::common_conditions::input_just_pressed, prelude::*};

use super::{
    grid::HexGrid,
    hex::{HexCoord, HEX_SIZE},
    projectile::{LEFT_WALL, RIGHT_WALL, TOP_WALL, DANGER_LINE_Y},
    shooter::SHOOTER_Y,
};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DebugGridVisible>();

    // Toggle debug with 'D' key
    app.add_systems(
        Update,
        toggle_debug
            .run_if(in_state(Screen::Gameplay).and(input_just_pressed(KeyCode::KeyD))),
    );

    // Draw debug grid when visible
    app.add_systems(
        Update,
        draw_debug_grid.run_if(in_state(Screen::Gameplay).and(debug_visible)),
    );

    // Always draw walls during gameplay
    app.add_systems(
        Update,
        draw_walls.run_if(in_state(Screen::Gameplay)),
    );
}

/// Resource to track if debug visualization is visible.
#[derive(Resource, Default)]
pub struct DebugGridVisible(pub bool);

fn debug_visible(debug: Res<DebugGridVisible>) -> bool {
    debug.0
}

fn toggle_debug(mut debug: ResMut<DebugGridVisible>) {
    debug.0 = !debug.0;
    let state = if debug.0 { "ON" } else { "OFF" };
    info!("Debug grid: {}", state);
}

/// Draw the debug grid using Bevy's Gizmos.
fn draw_debug_grid(mut gizmos: Gizmos, grid: Res<HexGrid>) {
    let bounds = &grid.bounds;

    // Draw all valid hex cells
    for r in bounds.min_r..=bounds.max_r {
        // In axial coordinates, q range is the same for all rows
        for q in bounds.min_q..=bounds.max_q {
            let coord = HexCoord::new(q, r);
            let is_occupied = grid.is_occupied(coord);

            // Choose color based on state
            let color = if is_occupied {
                css::LIMEGREEN.with_alpha(0.5)
            } else if r == 0 {
                // Top row (anchor row) in different color
                css::GOLD.with_alpha(0.3)
            } else if r >= bounds.max_r - 1 {
                // Danger zone near bottom
                css::INDIAN_RED.with_alpha(0.3)
            } else {
                css::WHITE.with_alpha(0.15)
            };

            draw_hex_outline(&mut gizmos, coord, HEX_SIZE, color);
        }
    }

    // Draw grid bounds outline
    draw_bounds_outline(&mut gizmos, bounds, HEX_SIZE);
}

/// Draw a hexagon outline at the given coordinates.
fn draw_hex_outline(gizmos: &mut Gizmos, coord: HexCoord, size: f32, color: impl Into<Color>) {
    let corners = coord.corners(size);
    let color = color.into();

    for i in 0..6 {
        let next = (i + 1) % 6;
        gizmos.line_2d(corners[i], corners[next], color);
    }
}

/// Draw the outer bounds of the grid.
fn draw_bounds_outline(
    gizmos: &mut Gizmos,
    bounds: &super::grid::GridBounds,
    size: f32,
) {
    let color = css::AQUA.with_alpha(0.8);

    // Top edge
    {
        let r = bounds.min_r;
        for q in bounds.min_q..=bounds.max_q {
            let coord = HexCoord::new(q, r);
            let corners = coord.corners(size);
            // Top-left to top-right edge (corners 1 and 2 for pointy-top)
            gizmos.line_2d(corners[1], corners[2], color);
        }
    }

    // Draw left and right edges
    for r in bounds.min_r..=bounds.max_r {
        // Left edge hex
        let left = HexCoord::new(bounds.min_q, r);
        let left_corners = left.corners(size);
        gizmos.line_2d(left_corners[3], left_corners[4], color); // West edge

        // Right edge hex
        let right = HexCoord::new(bounds.max_q, r);
        let right_corners = right.corners(size);
        gizmos.line_2d(right_corners[0], right_corners[5], color); // East edge
    }

    // Bottom edge (danger line)
    {
        let r = bounds.max_r;
        for q in bounds.min_q..=bounds.max_q {
            let coord = HexCoord::new(q, r);
            let corners = coord.corners(size);
            // Bottom edge (corners 4 and 5 for pointy-top)
            gizmos.line_2d(corners[4], corners[5], css::INDIAN_RED);
        }
    }
}

/// Draw the walls and play area boundaries (always visible during gameplay).
fn draw_walls(mut gizmos: Gizmos) {
    let wall_color = css::ORANGE.with_alpha(0.8);
    let danger_color = css::RED.with_alpha(0.6);

    // Left wall
    gizmos.line_2d(
        Vec2::new(LEFT_WALL, SHOOTER_Y - 50.0),
        Vec2::new(LEFT_WALL, TOP_WALL + 50.0),
        wall_color,
    );

    // Right wall
    gizmos.line_2d(
        Vec2::new(RIGHT_WALL, SHOOTER_Y - 50.0),
        Vec2::new(RIGHT_WALL, TOP_WALL + 50.0),
        wall_color,
    );

    // Top wall
    gizmos.line_2d(
        Vec2::new(LEFT_WALL, TOP_WALL),
        Vec2::new(RIGHT_WALL, TOP_WALL),
        wall_color,
    );

    // Danger line (game over zone)
    gizmos.line_2d(
        Vec2::new(LEFT_WALL, DANGER_LINE_Y),
        Vec2::new(RIGHT_WALL, DANGER_LINE_Y),
        danger_color,
    );
}
