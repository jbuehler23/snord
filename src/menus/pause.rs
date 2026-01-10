//! The pause menu.

use bevy::{ecs::spawn::SpawnWith, input::common_conditions::input_just_pressed, prelude::*};

use crate::{menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Pause).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let paused_header = asset_server.load("images/paused.png");
    let play_button = asset_server.load("images/play_button.png");
    let settings_button = asset_server.load("images/settings_button.png");
    let exit_button = asset_server.load("images/exit_button.png");

    commands.spawn((
        Name::new("Pause Menu"),
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
        // Semi-transparent background to hide game
        BackgroundColor(Color::srgba(0.96, 0.92, 0.84, 0.95)),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Pause),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Paused header image
            parent.spawn((
                Name::new("Paused Header"),
                ImageNode::new(paused_header),
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(160.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));
            parent.spawn(widget::button_image(play_button, 266.0, 105.0, close_menu));
            parent.spawn(widget::button_image(
                settings_button,
                266.0,
                105.0,
                open_settings_menu,
            ));
            parent.spawn(widget::button_image(
                exit_button,
                266.0,
                105.0,
                quit_to_title,
            ));
        })),
    ));
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
