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
        debug::plugin,
    ));
}

/// System to spawn the game level when entering gameplay.
/// Called from `screens/gameplay.rs` on `OnEnter(Screen::Gameplay)`.
pub fn spawn_game(mut commands: Commands) {
    commands.spawn((
        Name::new("Game"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
    ));

    info!("Game spawned - bubble shooter ready!");
}
