//! The settings menu.
//!
//! Additional settings and accessibility options should go here.

use bevy::{audio::Volume, ecs::spawn::SpawnWith, input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    menus::Menu,
    screens::Screen,
    theme::{GameFont, interaction::ImageInteractionPalette, palette::LABEL_TEXT, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Settings).and(input_just_pressed(KeyCode::Escape))),
    );

    app.add_systems(
        Update,
        update_global_volume_label.run_if(in_state(Menu::Settings)),
    );
}

fn spawn_settings_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_font: Res<GameFont>,
) {
    let settings_title = asset_server.load("images/settings_title.png");
    let back_button = asset_server.load("images/back_button.png");
    let minus_button = asset_server.load("images/minus_button.png");
    let plus_button = asset_server.load("images/plus_button.png");
    let font = game_font.0.clone();

    commands.spawn((
        Name::new("Settings Menu"),
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
        // Solid off-white background (same as main menu/splash)
        BackgroundColor(Color::srgb(0.96, 0.92, 0.84)),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Settings),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Settings title image
            parent.spawn((
                Name::new("Settings Title"),
                ImageNode::new(settings_title),
                Node {
                    width: Val::Px(500.0),
                    height: Val::Px(200.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Volume control row
            parent.spawn((
                Name::new("Volume Row"),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(15.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            )).with_children(|row| {
                // Volume label
                row.spawn((
                    Name::new("Volume Label"),
                    Text::new("Volume"),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(LABEL_TEXT),
                ));

                // Minus button
                row.spawn((
                    Name::new("Minus Button"),
                    Button,
                    ImageNode::new(minus_button),
                    ImageInteractionPalette {
                        none: Color::WHITE,
                        hovered: Color::srgb(0.85, 0.85, 0.85),
                        pressed: Color::srgb(0.7, 0.7, 0.7),
                    },
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(35.0),
                        ..default()
                    },
                )).observe(lower_global_volume);

                // Volume value
                row.spawn((
                    Name::new("Volume Value"),
                    Text::new("100%"),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(LABEL_TEXT),
                    GlobalVolumeLabel,
                    Node {
                        width: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                ));

                // Plus button
                row.spawn((
                    Name::new("Plus Button"),
                    Button,
                    ImageNode::new(plus_button),
                    ImageInteractionPalette {
                        none: Color::WHITE,
                        hovered: Color::srgb(0.85, 0.85, 0.85),
                        pressed: Color::srgb(0.7, 0.7, 0.7),
                    },
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(35.0),
                        ..default()
                    },
                )).observe(raise_global_volume);
            });

            // Back button
            parent.spawn(widget::button_image(back_button, 266.0, 105.0, go_back_on_click));
        })),
    ));
}

const MIN_VOLUME: f32 = 0.0;
const MAX_VOLUME: f32 = 3.0;

fn lower_global_volume(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() - 0.1).max(MIN_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

fn raise_global_volume(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() + 0.1).min(MAX_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GlobalVolumeLabel;

fn update_global_volume_label(
    global_volume: Res<GlobalVolume>,
    mut label: Single<&mut Text, With<GlobalVolumeLabel>>,
) {
    let percent = 100.0 * global_volume.volume.to_linear();
    label.0 = format!("{percent:3.0}%");
}

fn go_back_on_click(
    _: On<Pointer<Click>>,
    screen: Res<State<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

fn go_back(screen: Res<State<Screen>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}
