//! The power-up selection menu shown every 5 levels.

use bevy::{ecs::spawn::SpawnWith, prelude::*};

use crate::{
    game::powerups::{PowerUp, PowerUpChoices, UnlockedPowerUps},
    menus::Menu,
    theme::{GameFont, interaction::ImageInteractionPalette, palette::*},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::PowerUpSelect), spawn_powerup_menu);
}

/// Marker for power-up button to identify which power-up it represents.
#[derive(Component)]
struct PowerUpButton(PowerUp);

fn spawn_powerup_menu(
    mut commands: Commands,
    choices: Res<PowerUpChoices>,
    asset_server: Res<AssetServer>,
    game_font: Res<GameFont>,
) {
    let level = choices.level;
    let power_choices = choices.choices.clone();
    let button_template = asset_server.load("images/button_template.png");
    let font = game_font.0.clone();

    commands.spawn((
        Name::new("Power-Up Selection Menu"),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        // Semi-transparent background to hide game
        BackgroundColor(Color::srgba(0.96, 0.92, 0.84, 0.95)),
        Pickable::IGNORE,
        GlobalZIndex(2),
        DespawnOnExit(Menu::PowerUpSelect),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Header
            parent.spawn((
                Name::new("Header"),
                Text(format!("Level {level} - Choose Your Power!")),
                TextFont {
                    font: font.clone(),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(HEADER_TEXT),
            ));

            // Spawn buttons for each power-up choice
            for power in &power_choices {
                spawn_powerup_button(parent, *power, button_template.clone(), font.clone());
            }
        })),
    ));
}

fn spawn_powerup_button(
    parent: &mut ChildSpawner,
    power: PowerUp,
    button_image: Handle<Image>,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Name::new(format!("PowerUp Button: {}", power.name())),
            Node::default(),
        ))
        .with_children(|button_parent| {
            button_parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    PowerUpButton(power),
                    ImageNode::new(button_image),
                    ImageInteractionPalette {
                        none: Color::WHITE,
                        hovered: Color::srgb(0.85, 0.85, 0.85),
                        pressed: Color::srgb(0.7, 0.7, 0.7),
                    },
                    Node {
                        width: Val::Px(380.0),
                        height: Val::Px(150.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .with_children(|inner| {
                    // Power-up name
                    inner.spawn((
                        Text(power.name().to_string()),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                        Pickable::IGNORE,
                    ));
                    // Power-up description
                    inner.spawn((
                        Text(power.description().to_string()),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.3, 0.3)),
                        Pickable::IGNORE,
                    ));
                })
                .observe(select_powerup);
        });
}

fn select_powerup(
    trigger: On<Pointer<Click>>,
    button_query: Query<&PowerUpButton>,
    mut unlocked: ResMut<UnlockedPowerUps>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    if let Ok(power_button) = button_query.get(trigger.entity) {
        unlocked.add(power_button.0);
        next_menu.set(Menu::None);
    }
}
