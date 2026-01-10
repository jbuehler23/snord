//! Game state management - score, win/lose conditions, level progression.
//!
//! Win: Clear all bubbles from the grid.
//! Lose: Bubbles reach the danger zone (bottom of grid).
//!
//! Level system: After X shots, all bubbles descend and a new row spawns.

use bevy::prelude::*;

use super::{
    bubble::{Bubble, BubbleColor, GameAssets, spawn_bubble},
    cluster::{ClusterPopped, FloatingBubblesRemoved},
    grid::HexGrid,
    hex::{GridOffset, HEX_SIZE, HexCoord},
    highscore::{HighScores, ScoreEntry},
    powerups::{PowerUp, PowerUpChoices, UnlockedPowerUps},
    projectile::BubbleInDangerZone,
    shooter::SHOOTER_Y,
};
use crate::{PausableSystems, Pause, menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameScore>();
    app.init_resource::<GameLevel>();
    app.register_type::<GameScore>();
    app.register_type::<GameLevel>();

    app.add_message::<TriggerDescent>();

    app.add_systems(
        OnEnter(Screen::Gameplay),
        (reset_score, reset_level, reset_powerups, spawn_score_ui),
    );

    app.add_systems(
        Update,
        (
            update_score,
            update_score_ui,
            handle_descent,
            check_win_condition,
            check_lose_condition,
            check_danger_zone_game_over,
        )
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Marker component for the score text UI.
#[derive(Component)]
struct ScoreText;

/// Message to trigger bubble descent.
#[derive(Message, Debug, Clone)]
pub struct TriggerDescent;

/// Resource tracking the current level and descent timing.
#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct GameLevel {
    /// Current level number (starts at 1).
    pub level: u32,
    /// Number of shots before next descent.
    pub shots_until_descent: u32,
    /// Shots fired since last descent.
    pub shots_this_round: u32,
}

impl Default for GameLevel {
    fn default() -> Self {
        Self {
            level: 1,
            shots_until_descent: 8,
            shots_this_round: 0,
        }
    }
}

impl GameLevel {
    pub fn reset(&mut self) {
        self.level = 1;
        self.shots_until_descent = 8;
        self.shots_this_round = 0;
    }

    /// Called after each descent to advance the level.
    pub fn advance_level(&mut self) {
        self.level += 1;
        self.shots_this_round = 0;
        // Ramp down every 10 levels: 8 -> 7 -> 6 -> 5 (minimum)
        self.shots_until_descent = 8u32.saturating_sub(self.level / 10).max(5);
    }

    /// Returns shots remaining until next descent.
    pub fn shots_remaining(&self) -> u32 {
        self.shots_until_descent
            .saturating_sub(self.shots_this_round)
    }
}

/// Points awarded per bubble popped in a cluster.
const POINTS_PER_BUBBLE: u32 = 10;

/// Bonus multiplier for floating bubbles.
const FLOATING_BONUS_MULTIPLIER: u32 = 2;

/// The Y position below which bubbles trigger game over.
const DANGER_LINE_Y: f32 = SHOOTER_Y + 40.0;

/// Resource tracking the current game score.
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct GameScore {
    pub score: u32,
    pub bubbles_popped: u32,
    pub clusters_popped: u32,
}

impl GameScore {
    pub fn reset(&mut self) {
        self.score = 0;
        self.bubbles_popped = 0;
        self.clusters_popped = 0;
    }
}

/// Reset score when starting a new game.
fn reset_score(mut score: ResMut<GameScore>) {
    score.reset();
    info!("Score reset");
}

/// Reset level when starting a new game.
fn reset_level(mut level: ResMut<GameLevel>) {
    level.reset();
    info!("Level reset to 1");
}

/// Reset power-ups when starting a new game.
fn reset_powerups(mut powerups: ResMut<UnlockedPowerUps>) {
    powerups.reset();
}

/// Handle bubble descent when triggered.
fn handle_descent(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut grid: ResMut<HexGrid>,
    mut level: ResMut<GameLevel>,
    mut grid_offset: ResMut<GridOffset>,
    mut bubble_query: Query<(&Bubble, &mut Transform)>,
    mut descent_events: MessageReader<TriggerDescent>,
    mut danger_events: MessageWriter<BubbleInDangerZone>,
    // Power-up system
    unlocked_powerups: Res<UnlockedPowerUps>,
    mut powerup_choices: ResMut<PowerUpChoices>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut next_pause: ResMut<NextState<Pause>>,
    game_assets: Res<GameAssets>,
) {
    // Only process if we received a descent trigger
    if descent_events.read().next().is_none() {
        return;
    }

    info!("Descent triggered! Moving grid down...");

    // Move grid down by one row height (bubbles keep their coordinates)
    grid_offset.y -= HEX_SIZE * 1.5;

    // Update all bubble transforms with new offset (coords stay the same)
    for (_coord, &entity) in grid.iter() {
        if let Ok((bubble, mut transform)) = bubble_query.get_mut(entity) {
            let new_pos = bubble.coord.to_pixel_with_offset(HEX_SIZE, grid_offset.y);
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
        }
    }

    // Find the current minimum row to spawn new row above it
    let min_r = grid.iter().map(|(coord, _)| coord.r).min().unwrap_or(0);
    let new_row_r = min_r - 1;

    // Spawn new row at top
    let bounds = grid.bounds;
    for q in bounds.min_q..=bounds.max_q {
        let coord = HexCoord::new(q, new_row_r);
        let color = BubbleColor::random();
        let entity = spawn_bubble(
            &mut commands,
            &mut meshes,
            &mut materials,
            coord,
            color,
            grid_offset.y,
            Some(&game_assets),
        );
        grid.insert(coord, entity);
    }

    // Check for game over (any bubble below danger line after descent)
    for (_coord, &entity) in grid.iter() {
        if let Ok((_, transform)) = bubble_query.get(entity)
            && transform.translation.y < DANGER_LINE_Y
        {
            info!(
                "GAME OVER! Descent pushed bubble into danger zone at y={}",
                transform.translation.y
            );
            danger_events.write(BubbleInDangerZone);
            return;
        }
    }

    // Advance level
    level.advance_level();
    info!(
        "Level {} - next descent in {} shots (grid_offset.y = {})",
        level.level, level.shots_until_descent, grid_offset.y
    );

    // Check for power-up milestone (every 5 levels)
    if level.level > 0 && level.level.is_multiple_of(5) {
        let choices = PowerUp::random_choices(level.level, &unlocked_powerups.powers);
        if !choices.is_empty() {
            info!("Power-up selection at level {}!", level.level);
            powerup_choices.choices = choices;
            powerup_choices.level = level.level;
            next_pause.set(Pause(true));
            next_menu.set(Menu::PowerUpSelect);
        }
    }
}

/// Spawn the score text UI.
fn spawn_score_ui(mut commands: Commands, game_font: Res<crate::theme::GameFont>) {
    commands.spawn((
        ScoreText,
        Text::new("Score: 0"),
        TextFont {
            font: game_font.0.clone(),
            font_size: 20.0,
            ..default()
        },
        TextLayout::new_with_justify(bevy::text::Justify::Center),
        // Black text for light background
        TextColor(Color::srgb(0.1, 0.1, 0.1)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        DespawnOnExit(Screen::Gameplay),
    ));
}

/// Update the score text when score or level changes.
fn update_score_ui(
    score: Res<GameScore>,
    level: Res<GameLevel>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if !score.is_changed() && !level.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!(
            "Score: {}     Level: {}     Shots: {}",
            score.score,
            level.level,
            level.shots_remaining()
        );
    }
}

/// Update score when clusters/floating bubbles are removed.
fn update_score(
    mut score: ResMut<GameScore>,
    mut cluster_events: MessageReader<ClusterPopped>,
    mut floating_events: MessageReader<FloatingBubblesRemoved>,
    powerups: Res<UnlockedPowerUps>,
) {
    for event in cluster_events.read() {
        let mut points = event.count as u32 * POINTS_PER_BUBBLE;

        // Combo Snord: +50% score bonus for clusters larger than 3
        if powerups.has(PowerUp::ComboSnord) && event.count > 3 {
            let bonus = points / 2; // 50% bonus
            points += bonus;
            info!(
                "Combo Snord bonus! +{} extra points for cluster of {}",
                bonus, event.count
            );
        }

        score.score += points;
        score.bubbles_popped += event.count as u32;
        score.clusters_popped += 1;

        info!(
            "Cluster popped: {} {:?} bubbles, +{} points (total: {})",
            event.count, event.color, points, score.score
        );
    }

    for event in floating_events.read() {
        let points = event.count as u32 * POINTS_PER_BUBBLE * FLOATING_BONUS_MULTIPLIER;
        score.score += points;
        score.bubbles_popped += event.count as u32;

        info!(
            "Floating bubbles removed: {}, +{} bonus points (total: {})",
            event.count, points, score.score
        );
    }
}

/// Check if the player has won (all bubbles cleared).
fn check_win_condition(
    grid: Res<HexGrid>,
    mut next_menu: ResMut<NextState<Menu>>,
    score: Res<GameScore>,
    mut high_scores: ResMut<HighScores>,
) {
    // Need to have popped at least one cluster to win
    // (prevents winning on empty grid at start)
    if score.clusters_popped > 0 && grid.is_empty() {
        info!("WIN! All bubbles cleared! Final score: {}", score.score);

        // Save high score if it qualifies
        let entry = ScoreEntry::new(score.score, score.bubbles_popped);
        if high_scores.add_score(entry) {
            info!("New high score!");
            high_scores.save();
        }

        // Show win screen (using credits menu as placeholder)
        next_menu.set(Menu::Credits);
    }
}

/// Check if the player has lost (bubbles too low).
fn check_lose_condition(
    grid: Res<HexGrid>,
    bubble_query: Query<&Transform, With<Bubble>>,
    mut next_menu: ResMut<NextState<Menu>>,
    score: Res<GameScore>,
    mut high_scores: ResMut<HighScores>,
) {
    // Check if any bubble is below the danger line
    for (_coord, &entity) in grid.iter() {
        if let Ok(transform) = bubble_query.get(entity)
            && transform.translation.y < DANGER_LINE_Y
        {
            info!(
                "GAME OVER! Bubble reached danger zone. Final score: {}",
                score.score
            );

            // Save high score if it qualifies
            let entry = ScoreEntry::new(score.score, score.bubbles_popped);
            if high_scores.add_score(entry) {
                info!("New high score!");
                high_scores.save();
            }

            // Show game over screen
            next_menu.set(Menu::GameOver);
            return;
        }
    }
}

/// Check for game over triggered by projectile landing in danger zone.
fn check_danger_zone_game_over(
    mut danger_events: MessageReader<BubbleInDangerZone>,
    mut next_menu: ResMut<NextState<Menu>>,
    score: Res<GameScore>,
    mut high_scores: ResMut<HighScores>,
) {
    for _ in danger_events.read() {
        info!(
            "GAME OVER! Projectile tried to land in danger zone. Final score: {}",
            score.score
        );

        // Save high score if it qualifies
        let entry = ScoreEntry::new(score.score, score.bubbles_popped);
        if high_scores.add_score(entry) {
            info!("New high score!");
            high_scores.save();
        }

        // Show game over screen
        next_menu.set(Menu::GameOver);
    }
}
