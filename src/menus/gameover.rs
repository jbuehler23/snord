//! The game over menu.

use bevy::prelude::*;

use crate::{menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::GameOver), spawn_gameover_menu);
}

fn spawn_gameover_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Game Over Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::GameOver),
        children![
            widget::header("Game Over"),
            widget::button("Settings", open_settings_menu),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
