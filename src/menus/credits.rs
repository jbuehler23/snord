//! The credits menu.

use bevy::{ecs::spawn::SpawnWith, input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    menus::Menu,
    theme::{GameFont, palette::HEADER_TEXT, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_credits_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_font: Res<GameFont>,
) {
    let back_button = asset_server.load("images/back_button.png");
    let font = game_font.0.clone();

    commands.spawn((
        Name::new("Credits Menu"),
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
        BackgroundColor(Color::srgb(0.96, 0.92, 0.84)),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Credits),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Header
            parent.spawn((
                Name::new("Credits Header"),
                Text::new("Credits"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(HEADER_TEXT),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Created by section
            parent.spawn((
                Text::new("Created by"),
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(HEADER_TEXT),
            ));
            parent.spawn((
                Text::new("Joe :)"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(15.0)),
                    ..default()
                },
            ));

            // Assets section
            parent.spawn((
                Text::new("Assets"),
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(HEADER_TEXT),
            ));
            parent.spawn((
                Text::new("SFX: Joe's Mouth"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
            ));
            parent.spawn((
                Text::new("Art: Joe's Hand"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(15.0)),
                    ..default()
                },
            ));

            // Made with Bevy
            parent.spawn((
                Text::new("Made with Bevy"),
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(HEADER_TEXT),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Back button
            parent.spawn(widget::button_image(back_button, 266.0, 105.0, go_back_on_click));
        })),
    ));
}

fn go_back_on_click(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}
