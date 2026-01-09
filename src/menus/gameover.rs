//! The game over menu.

use bevy::prelude::*;

use crate::{
    menus::Menu,
    screens::Screen,
    theme::{palette::HEADER_TEXT, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::GameOver), spawn_gameover_menu);
}

fn spawn_gameover_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let play_button = asset_server.load("images/play_button.png");
    let settings_button = asset_server.load("images/settings_button.png");
    let exit_button = asset_server.load("images/exit_button.png");

    commands.spawn((
        Name::new("Game Over Menu"),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },
        // Semi-transparent background
        BackgroundColor(Color::srgba(0.96, 0.92, 0.84, 0.95)),
        GlobalZIndex(2),
        DespawnOnExit(Menu::GameOver),
        children![
            (
                Name::new("Game Over Header"),
                Text("Game Over".to_string()),
                TextFont::from_font_size(48.0),
                TextColor(HEADER_TEXT),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ),
            widget::button_image(play_button, 266.0, 105.0, restart_game),
            widget::button_image(settings_button, 266.0, 105.0, open_settings_menu),
            widget::button_image(exit_button, 266.0, 105.0, quit_to_title),
        ],
    ));
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn restart_game(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    // Restart by re-entering the Gameplay screen (triggers reset systems)
    next_screen.set(Screen::Gameplay);
}
