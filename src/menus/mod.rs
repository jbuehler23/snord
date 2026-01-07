//! The game's menus and transitions between them.

mod credits;
mod gameover;
mod main;
mod pause;
mod powerup_select;
mod settings;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();

    app.add_plugins((
        credits::plugin,
        gameover::plugin,
        main::plugin,
        pause::plugin,
        powerup_select::plugin,
        settings::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Menu {
    #[default]
    None,
    Main,
    Credits,
    Settings,
    Pause,
    GameOver,
    PowerUpSelect,
}
