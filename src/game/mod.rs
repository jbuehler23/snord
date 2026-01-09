//! The main game module for the bubble shooter.
//!
//! This module contains all the gameplay logic including:
//! - Hexagonal grid system (axial coordinates)
//! - Bubble entities and colors
//! - Shooter/launcher mechanics
//! - Projectile physics
//! - Cluster detection and popping
//! - Game state management

mod bubble;
mod cluster;
mod debug;
mod grid;
mod hex;
mod highscore;
mod polish;
pub mod powerups;
mod projectile;
mod shooter;
mod state;

use bevy::prelude::*;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        hex::plugin,
        grid::plugin,
        bubble::plugin,
        shooter::plugin,
        projectile::plugin,
        cluster::plugin,
        state::plugin,
        highscore::plugin,
        powerups::plugin,
        polish::plugin,
        debug::plugin,
    ));
}

/// System to spawn the game level when entering gameplay.
/// Called from `screens/gameplay.rs` on `OnEnter(Screen::Gameplay)`.
pub fn spawn_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Game"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
    ));

    // Spawn game panel background (centered on playfield)
    // Playfield: TOP_WALL=280, SHOOTER_Y=-250, so center Y = (280 + (-250)) / 2 = 15
    let panel_image = asset_server.load("images/game_bounds.png");
    commands.spawn((
        Name::new("Game Panel"),
        Sprite::from_image(panel_image),
        Transform::from_xyz(0.0, 15.0, -1.0), // Z=-1 to be behind bubbles
        DespawnOnExit(Screen::Gameplay),
    ));

    // Spawn danger line indicator (Y=-170, overlays game panel)
    let danger_line_image = asset_server.load("images/danger_line.png");
    commands.spawn((
        Name::new("Danger Line"),
        Sprite::from_image(danger_line_image),
        Transform::from_xyz(0.0, -170.0, 0.0), // Z=0 to overlay game panel
        DespawnOnExit(Screen::Gameplay),
    ));

    info!("Game spawned - bubble shooter ready!");
}
