//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{asset_tracking::ResourceHandles, menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let title = asset_server.load("images/title.png");
    let play_button = asset_server.load("images/play_button.png");
    let settings_button = asset_server.load("images/settings_button.png");
    let credits_button = asset_server.load("images/credits_button.png");
    #[cfg(not(target_family = "wasm"))]
    let exit_button = asset_server.load("images/exit_button.png");

    commands.spawn((
        Name::new("Main Menu"),
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
        GlobalZIndex(2),
        DespawnOnExit(Menu::Main),
        #[cfg(not(target_family = "wasm"))]
        children![
            (
                Name::new("Title"),
                ImageNode::new(title.clone()),
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(120.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ),
            widget::button_image(play_button.clone(), 266.0, 105.0, enter_loading_or_gameplay_screen),
            widget::button_image(settings_button.clone(), 266.0, 105.0, open_settings_menu),
            widget::button_image(credits_button.clone(), 266.0, 105.0, open_credits_menu),
            widget::button_image(exit_button.clone(), 266.0, 105.0, exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            (
                Name::new("Title"),
                ImageNode::new(title),
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(120.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ),
            widget::button_image(play_button, 266.0, 105.0, enter_loading_or_gameplay_screen),
            widget::button_image(settings_button, 266.0, 105.0, open_settings_menu),
            widget::button_image(credits_button, 266.0, 105.0, open_credits_menu),
        ],
    ));
}

fn enter_loading_or_gameplay_screen(
    _: On<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<Pointer<Click>>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
