//! Hexagonal coordinate system using offset coordinates (odd-r).
//!
//! Based on Red Blob Games' excellent guide:
//! https://www.redblobgames.com/grids/hexagons/
//!
//! We use "pointy-top" orientation with "odd-r" offset coordinates.
//! This creates a rectangular grid where odd rows are shifted right by half a hex.
//! This is the classic bubble shooter layout.

use bevy::prelude::*;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<HexCoord>();
    app.register_type::<GridOffset>();
    app.init_resource::<GridOffset>();
    app.add_systems(OnEnter(Screen::Gameplay), reset_grid_offset);
}

/// Resource tracking the grid's Y origin (decreases on descent).
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct GridOffset {
    pub y: f32,
}

impl Default for GridOffset {
    fn default() -> Self {
        Self { y: GRID_ORIGIN_Y }
    }
}

/// Reset grid offset when starting a new game.
fn reset_grid_offset(mut grid_offset: ResMut<GridOffset>) {
    grid_offset.y = GRID_ORIGIN_Y;
    info!("Grid offset reset to {}", grid_offset.y);
}

/// Square root of 3, used frequently in hex math.
pub const SQRT_3: f32 = 1.732_050_8;

/// The size (outer radius) of each hexagon in pixels.
/// This is the distance from center to vertex.
pub const HEX_SIZE: f32 = 20.0;

/// The Y offset to position the grid properly on screen.
/// Row 0 will be at this Y position.
pub const GRID_ORIGIN_Y: f32 = 250.0;

/// Offset hex coordinate (odd-r system).
///
/// In offset coordinates:
/// - q is the column (increases to the right)
/// - r is the row (increases downward)
/// - Odd rows are shifted right by half a hex width
///
/// This creates a rectangular grid appearance, perfect for bubble shooters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct HexCoord {
    /// Column (x-axis)
    pub q: i32,
    /// Row (y-axis)
    pub r: i32,
}

impl HexCoord {
    /// Create a new hex coordinate.
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// The origin hex at (0, 0).
    #[allow(dead_code)]
    pub const ORIGIN: Self = Self { q: 0, r: 0 };

    /// Get the derived s coordinate (cube coordinates constraint: q + r + s = 0).
    #[inline]
    #[allow(dead_code)]
    pub const fn s(&self) -> i32 {
        -self.q - self.r
    }

    /// Get all 6 neighboring hex coordinates.
    ///
    /// In offset coordinates (odd-r), neighbors depend on row parity.
    /// Odd rows are shifted right, so neighbor offsets differ.
    pub fn neighbors(&self) -> [HexCoord; 6] {
        // Odd-r offset coordinate neighbor directions
        // Even rows and odd rows have different offsets for diagonal neighbors
        let is_odd_row = self.r % 2 != 0;

        if is_odd_row {
            // Odd row (shifted right)
            [
                HexCoord::new(self.q + 1, self.r),     // East
                HexCoord::new(self.q + 1, self.r - 1), // Northeast
                HexCoord::new(self.q, self.r - 1),     // Northwest
                HexCoord::new(self.q - 1, self.r),     // West
                HexCoord::new(self.q, self.r + 1),     // Southwest
                HexCoord::new(self.q + 1, self.r + 1), // Southeast
            ]
        } else {
            // Even row (not shifted)
            [
                HexCoord::new(self.q + 1, self.r),     // East
                HexCoord::new(self.q, self.r - 1),     // Northeast
                HexCoord::new(self.q - 1, self.r - 1), // Northwest
                HexCoord::new(self.q - 1, self.r),     // West
                HexCoord::new(self.q - 1, self.r + 1), // Southwest
                HexCoord::new(self.q, self.r + 1),     // Southeast
            ]
        }
    }

    /// Calculate the hex distance between two coordinates.
    ///
    /// In cube coordinates, this is: max(|dq|, |dr|, |ds|)
    /// Or equivalently: (|dq| + |dr| + |ds|) / 2
    #[allow(dead_code)]
    pub fn distance(&self, other: HexCoord) -> i32 {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = (self.s() - other.s()).abs();
        (dq + dr + ds) / 2
    }

    /// Convert offset hex coordinates to pixel (world) position.
    ///
    /// For odd-r offset coordinates (pointy-top):
    /// - x = size * sqrt(3) * (q + 0.5 if odd row)
    /// - y = size * 1.5 * r
    ///
    /// Odd rows are shifted right by half a hex width, creating a rectangular grid.
    /// Uses the default GRID_ORIGIN_Y constant.
    pub fn to_pixel(&self, size: f32) -> Vec2 {
        self.to_pixel_with_offset(size, GRID_ORIGIN_Y)
    }

    /// Convert offset hex coordinates to pixel (world) position with custom grid origin.
    ///
    /// Use this version when the grid origin has changed (e.g., after descent).
    pub fn to_pixel_with_offset(&self, size: f32, grid_origin_y: f32) -> Vec2 {
        // Odd rows shift right by half a hex width
        let row_offset = if self.r % 2 != 0 { 0.5 } else { 0.0 };
        let x = size * SQRT_3 * (self.q as f32 + row_offset);
        let y = size * 1.5 * self.r as f32;
        Vec2::new(x, grid_origin_y - y)
    }

    /// Convert pixel (world) position to offset hex coordinates.
    ///
    /// This returns the nearest hex to the given position.
    /// For offset coordinates, we find the row first, then determine column
    /// based on row parity (odd rows are shifted right).
    /// Uses the default GRID_ORIGIN_Y constant.
    #[allow(dead_code)]
    pub fn from_pixel(pos: Vec2, size: f32) -> Self {
        Self::from_pixel_with_offset(pos, size, GRID_ORIGIN_Y)
    }

    /// Convert pixel (world) position to offset hex coordinates with custom grid origin.
    ///
    /// Use this version when the grid origin has changed (e.g., after descent).
    pub fn from_pixel_with_offset(pos: Vec2, size: f32, grid_origin_y: f32) -> Self {
        // Account for grid origin offset
        let y = grid_origin_y - pos.y;
        let x = pos.x;

        // Find row first (simple division)
        let r = (y / (size * 1.5)).round() as i32;

        // Determine column offset based on row parity
        let row_offset = if r % 2 != 0 { 0.5 } else { 0.0 };

        // Find column with offset correction
        let q = (x / (size * SQRT_3) - row_offset).round() as i32;

        Self { q, r }
    }

    /// Get the 6 corner vertices of this hex in world coordinates.
    ///
    /// Useful for debug drawing. Returns corners in order for drawing a polygon.
    pub fn corners(&self, size: f32) -> [Vec2; 6] {
        let center = self.to_pixel(size);
        let mut corners = [Vec2::ZERO; 6];

        for i in 0..6 {
            // For pointy-top, first corner is at 30 degrees
            let angle = std::f32::consts::PI / 180.0 * (60.0 * i as f32 + 30.0);
            corners[i] = Vec2::new(center.x + size * angle.cos(), center.y + size * angle.sin());
        }

        corners
    }
}

impl std::fmt::Display for HexCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.q, self.r)
    }
}

impl std::ops::Add for HexCoord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        HexCoord::new(self.q + other.q, self.r + other.r)
    }
}

impl std::ops::Sub for HexCoord {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        HexCoord::new(self.q - other.q, self.r - other.r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbors_count() {
        let hex = HexCoord::new(0, 0);
        assert_eq!(hex.neighbors().len(), 6);
    }

    #[test]
    fn test_pixel_roundtrip_even_row() {
        let original = HexCoord::new(5, 2);
        let pixel = original.to_pixel(HEX_SIZE);
        let back = HexCoord::from_pixel(pixel, HEX_SIZE);
        assert_eq!(original, back);
    }

    #[test]
    fn test_pixel_roundtrip_odd_row() {
        let original = HexCoord::new(3, 3);
        let pixel = original.to_pixel(HEX_SIZE);
        let back = HexCoord::from_pixel(pixel, HEX_SIZE);
        assert_eq!(original, back);
    }
}
