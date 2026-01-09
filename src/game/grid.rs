//! The hexagonal grid that holds all bubbles.
//!
//! Uses a HashMap for sparse storage - only occupied cells are stored.
//! This is more flexible than a 2D array and handles the hex coordinate
//! system naturally.

use bevy::prelude::*;
use std::collections::HashMap;

use super::hex::{HEX_SIZE, HexCoord};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<HexGrid>();
    app.register_type::<HexGrid>();
    app.register_type::<GridBounds>();
}

/// The bounds of the playable grid area.
///
/// Defines which hex coordinates are valid for the game.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct GridBounds {
    /// Minimum q coordinate (left edge).
    pub min_q: i32,
    /// Maximum q coordinate (right edge).
    pub max_q: i32,
    /// Minimum r coordinate (top edge, typically 0).
    pub min_r: i32,
    /// Maximum r coordinate (bottom edge / danger zone).
    pub max_r: i32,
}

impl Default for GridBounds {
    fn default() -> Self {
        // Grid sized to match wall boundaries:
        // Hex width = HEX_SIZE * sqrt(3) = 20 * 1.732 ≈ 34.64px
        // For q = -6 to 6 (13 columns):
        //   Even rows: centers at -207.8 to 207.8, edges at ±225.1px
        //   Odd rows: centers at -190.5 to 225.1, edges at ±242.4px
        //   Walls at ±245px for margin
        //
        // Height: 14 rows, hex height = 1.5 * 20 = 30px
        Self {
            min_q: -6,
            max_q: 6,
            min_r: 0,
            max_r: 13,
        }
    }
}

impl GridBounds {
    /// Check if a hex coordinate is within bounds.
    pub fn contains(&self, coord: HexCoord) -> bool {
        coord.q >= self.min_q
            && coord.q <= self.max_q
            && coord.r >= self.min_r
            && coord.r <= self.max_r
    }

    /// Iterate over all valid hex coordinates in the grid.
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = HexCoord> {
        let min_q = self.min_q;
        let max_q = self.max_q;
        let min_r = self.min_r;
        let max_r = self.max_r;

        (min_r..=max_r).flat_map(move |r| {
            // In axial coordinates, q range is the same for all rows
            (min_q..=max_q).map(move |q| HexCoord::new(q, r))
        })
    }

    /// Get the number of columns for a given row.
    #[allow(dead_code)]
    pub fn columns_in_row(&self, _r: i32) -> i32 {
        self.max_q - self.min_q + 1
    }

    /// Get the center position in world coordinates.
    #[allow(dead_code)]
    pub fn center_world(&self) -> Vec2 {
        let center_r = (self.min_r + self.max_r) / 2;
        let center_q = (self.min_q + self.max_q) / 2;
        HexCoord::new(center_q, center_r).to_pixel(HEX_SIZE)
    }
}

/// The main grid resource holding all bubbles.
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct HexGrid {
    /// Map from hex coordinates to bubble entities.
    #[reflect(ignore)]
    bubbles: HashMap<HexCoord, Entity>,

    /// The playable area bounds.
    pub bounds: GridBounds,
}

impl HexGrid {
    /// Create a new empty grid with default bounds.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a cell is occupied.
    pub fn is_occupied(&self, coord: HexCoord) -> bool {
        self.bubbles.contains_key(&coord)
    }

    /// Check if a coordinate is adjacent to any occupied cell.
    fn is_adjacent_to_bubble(&self, coord: HexCoord) -> bool {
        coord.neighbors().iter().any(|n| self.is_occupied(*n))
    }

    /// Get the bubble entity at a position, if any.
    pub fn get(&self, coord: HexCoord) -> Option<Entity> {
        self.bubbles.get(&coord).copied()
    }

    /// Insert a bubble at a position.
    ///
    /// Returns the previous entity if the cell was occupied.
    pub fn insert(&mut self, coord: HexCoord, entity: Entity) -> Option<Entity> {
        self.bubbles.insert(coord, entity)
    }

    /// Remove a bubble from a position.
    ///
    /// Returns the entity that was removed, if any.
    pub fn remove(&mut self, coord: HexCoord) -> Option<Entity> {
        self.bubbles.remove(&coord)
    }

    /// Clear all bubbles from the grid.
    pub fn clear(&mut self) {
        self.bubbles.clear();
    }

    /// Get the number of bubbles in the grid.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.bubbles.len()
    }

    /// Check if the grid is empty.
    pub fn is_empty(&self) -> bool {
        self.bubbles.is_empty()
    }

    /// Iterate over all occupied cells.
    pub fn iter(&self) -> impl Iterator<Item = (&HexCoord, &Entity)> {
        self.bubbles.iter()
    }

    /// Get all occupied coordinates.
    pub fn coords(&self) -> impl Iterator<Item = HexCoord> + '_ {
        self.bubbles.keys().copied()
    }

    /// Find empty neighbors of occupied cells.
    ///
    /// Useful for finding where a projectile can snap to.
    #[allow(dead_code)]
    pub fn empty_neighbors(&self, coord: HexCoord) -> Vec<HexCoord> {
        coord
            .neighbors()
            .into_iter()
            .filter(|n| self.bounds.contains(*n) && !self.is_occupied(*n))
            .collect()
    }

    /// Find the closest empty cell to a world position.
    ///
    /// This is used when a projectile needs to snap to the grid.
    /// It first converts the position to hex coordinates, then finds
    /// the nearest valid empty cell.
    pub fn closest_empty_cell(&self, world_pos: Vec2, grid_origin_y: f32) -> Option<HexCoord> {
        let target = HexCoord::from_pixel_with_offset(world_pos, HEX_SIZE, grid_origin_y);

        // If the target cell is valid and empty, use it
        // Allow cells within bounds OR adjacent to existing bubbles (for descended rows)
        if (self.bounds.contains(target) || self.is_adjacent_to_bubble(target))
            && !self.is_occupied(target)
        {
            return Some(target);
        }

        // Otherwise, search neighbors in expanding rings
        let mut checked = std::collections::HashSet::new();
        let mut to_check = vec![target];

        while !to_check.is_empty() {
            let mut next_ring = Vec::new();

            for coord in to_check {
                if checked.contains(&coord) {
                    continue;
                }
                checked.insert(coord);

                // Allow cells within bounds OR adjacent to existing bubbles (for descended rows)
                if (self.bounds.contains(coord) || self.is_adjacent_to_bubble(coord))
                    && !self.is_occupied(coord)
                {
                    return Some(coord);
                }

                // Add neighbors for next iteration
                for neighbor in coord.neighbors() {
                    if !checked.contains(&neighbor) {
                        next_ring.push(neighbor);
                    }
                }
            }

            to_check = next_ring;

            // Safety limit to prevent infinite loops
            if checked.len() > 1000 {
                break;
            }
        }

        None
    }

    /// Get the lowest row (highest r value) that has bubbles.
    /// Used for checking game over condition.
    #[allow(dead_code)]
    pub fn lowest_row(&self) -> Option<i32> {
        self.bubbles.keys().map(|c| c.r).max()
    }

    /// Get all bubbles in the top row (smallest r value).
    /// Used as starting point for floating bubble detection.
    pub fn top_row_coords(&self) -> Vec<HexCoord> {
        // Find the minimum r value (top row may be negative after descents)
        let Some(min_r) = self.bubbles.keys().map(|c| c.r).min() else {
            return Vec::new();
        };

        self.bubbles
            .keys()
            .filter(|c| c.r == min_r)
            .copied()
            .collect()
    }
}
