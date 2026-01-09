//! Reusable UI widgets & theming.

// Unused utilities may trigger this lints undesirably.
#![allow(dead_code)]

pub mod interaction;
pub mod palette;
pub mod widget;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::{interaction::InteractionPalette, palette as ui_palette, widget};
}

use bevy::prelude::*;

/// Resource holding the game's custom font.
#[derive(Resource)]
pub struct GameFont(pub Handle<Font>);

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(interaction::plugin);
    app.add_systems(Startup, load_game_font);
}

fn load_game_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/RockSalt-Regular.ttf");
    commands.insert_resource(GameFont(font));
}
