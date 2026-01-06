//! Game state management - score, win/lose conditions, level progression.
//!
//! Win: Clear all bubbles from the grid.
//! Lose: Bubbles reach the danger zone (bottom of grid).
//!
//! Level system: After X shots, all bubbles descend and a new row spawns.

use bevy::prelude::*;

use super::{
    bubble::{spawn_bubble, Bubble, BubbleColor},
    cluster::{ClusterPopped, FloatingBubblesRemoved},
    grid::HexGrid,
    hex::{GridOffset, HexCoord, HEX_SIZE},
    highscore::{HighScores, ScoreEntry},
    projectile::BubbleInDangerZone,
    shooter::SHOOTER_Y,
};
use crate::{screens::Screen, PausableSystems, menus::Menu};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameScore>();
    app.init_resource::<GameLevel>();
    app.register_type::<GameScore>();
    app.register_type::<GameLevel>();

    app.add_message::<TriggerDescent>();

    app.add_systems(OnEnter(Screen::Gameplay), (reset_score, reset_level, spawn_score_ui));

    app.add_systems(
        Update,
        (
            update_score,
            update_score_ui,
            handle_descent,
            check_win_condition,
            check_lose_condition,
            check_danger_zone_game_over,
            draw_danger_line,
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
        // Decrease shots needed: 8 -> 7 -> 6 -> 5 (minimum)
        self.shots_until_descent = (9u32.saturating_sub(self.level)).max(5);
    }

    /// Returns shots remaining until next descent.
    pub fn shots_remaining(&self) -> u32 {
        self.shots_until_descent.saturating_sub(self.shots_this_round)
    }
}

/// Points awarded per bubble popped in a cluster.
const POINTS_PER_BUBBLE: u32 = 10;

/// Bonus multiplier for floating bubbles.
const FLOATING_BONUS_MULTIPLIER: u32 = 2;

/// The Y position below which bubbles trigger game over.
const DANGER_LINE_Y: f32 = SHOOTER_Y + 80.0;

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
        let entity = spawn_bubble(&mut commands, &mut meshes, &mut materials, coord, color, grid_offset.y);
        grid.insert(coord, entity);
    }

    // Check for game over (any bubble below danger line after descent)
    for (_coord, &entity) in grid.iter() {
        if let Ok((_, transform)) = bubble_query.get(entity) {
            if transform.translation.y < DANGER_LINE_Y {
                info!(
                    "GAME OVER! Descent pushed bubble into danger zone at y={}",
                    transform.translation.y
                );
                danger_events.write(BubbleInDangerZone);
                return;
            }
        }
    }

    // Advance level
    level.advance_level();
    info!(
        "Level {} - next descent in {} shots (grid_offset.y = {})",
        level.level, level.shots_until_descent, grid_offset.y
    );
}

/// Spawn the score text UI.
fn spawn_score_ui(mut commands: Commands) {
    commands.spawn((
        ScoreText,
        Text::new("Score: 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
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
            "Score: {}\nLevel: {}\nShots: {}",
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
) {
    for event in cluster_events.read() {
        let points = event.count as u32 * POINTS_PER_BUBBLE;
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
        if let Ok(transform) = bubble_query.get(entity) {
            if transform.translation.y < DANGER_LINE_Y {
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

/// Draw the danger line indicator.
fn draw_danger_line(mut gizmos: Gizmos) {
    let danger_y = DANGER_LINE_Y;
    gizmos.line_2d(
        Vec2::new(-400.0, danger_y),
        Vec2::new(400.0, danger_y),
        Color::srgba(1.0, 0.2, 0.2, 0.5),
    );
}
