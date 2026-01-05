//! Game state management - score, win/lose conditions.
//!
//! Win: Clear all bubbles from the grid.
//! Lose: Bubbles reach the danger zone (bottom of grid).

use bevy::prelude::*;

use super::{
    bubble::Bubble,
    cluster::{ClusterPopped, FloatingBubblesRemoved},
    grid::HexGrid,
    highscore::{HighScores, ScoreEntry},
    projectile::BubbleInDangerZone,
    shooter::SHOOTER_Y,
};
use crate::{screens::Screen, PausableSystems, menus::Menu};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameScore>();
    app.register_type::<GameScore>();

    app.add_systems(OnEnter(Screen::Gameplay), reset_score);

    app.add_systems(
        Update,
        (
            update_score,
            check_win_condition,
            check_lose_condition,
            check_danger_zone_game_over,
            draw_score_ui,
        )
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
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

                // Show game over (using pause menu as placeholder)
                next_menu.set(Menu::Pause);
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

        // Show game over (using pause menu as placeholder)
        next_menu.set(Menu::Pause);
    }
}

/// Draw the score UI.
fn draw_score_ui(mut gizmos: Gizmos, _score: Res<GameScore>) {
    // Draw score text at top of screen
    // Note: Using gizmos for now, could use proper UI later
    // TODO: Add proper text rendering for score display

    // Draw score background
    let score_pos = Vec2::new(0.0, 280.0);
    gizmos.rect_2d(
        Isometry2d::from_translation(score_pos),
        Vec2::new(200.0, 40.0),
        Color::srgba(0.0, 0.0, 0.0, 0.7),
    );

    // Draw danger line
    let danger_y = DANGER_LINE_Y;
    gizmos.line_2d(
        Vec2::new(-400.0, danger_y),
        Vec2::new(400.0, danger_y),
        Color::srgba(1.0, 0.2, 0.2, 0.5),
    );
}
