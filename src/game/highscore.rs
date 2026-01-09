//! High score persistence with Top 10 leaderboard.
//!
//! Scores are saved to a local JSON file in the user's data directory.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<HighScores>();

    // Load high scores on startup
    app.add_systems(Startup, load_high_scores);
}

/// Maximum number of high scores to keep.
const MAX_HIGH_SCORES: usize = 10;

/// A single high score entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub score: u32,
    pub bubbles_popped: u32,
}

impl ScoreEntry {
    pub fn new(score: u32, bubbles_popped: u32) -> Self {
        Self {
            score,
            bubbles_popped,
        }
    }
}

/// Resource holding the top 10 high scores.
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct HighScores {
    pub entries: Vec<ScoreEntry>,
}

impl HighScores {
    /// Check if a score would make it into the top 10.
    #[allow(dead_code)]
    pub fn is_high_score(&self, score: u32) -> bool {
        if score == 0 {
            return false;
        }
        if self.entries.len() < MAX_HIGH_SCORES {
            return true;
        }
        self.entries
            .last()
            .map(|lowest| score > lowest.score)
            .unwrap_or(true)
    }

    /// Add a new score to the leaderboard (if it qualifies).
    /// Returns true if the score was added.
    pub fn add_score(&mut self, entry: ScoreEntry) -> bool {
        if entry.score == 0 {
            return false;
        }

        // Insert in sorted position (descending by score)
        let pos = self
            .entries
            .iter()
            .position(|e| entry.score > e.score)
            .unwrap_or(self.entries.len());

        if pos >= MAX_HIGH_SCORES {
            return false;
        }

        self.entries.insert(pos, entry);

        // Trim to max size
        if self.entries.len() > MAX_HIGH_SCORES {
            self.entries.truncate(MAX_HIGH_SCORES);
        }

        true
    }

    /// Get the file path for storing high scores.
    fn file_path() -> Option<PathBuf> {
        dirs::data_local_dir().map(|dir| dir.join("snord").join("highscores.json"))
    }

    /// Load high scores from disk.
    pub fn load() -> Self {
        let Some(path) = Self::file_path() else {
            warn!("Could not determine data directory for high scores");
            return Self::default();
        };

        if !path.exists() {
            info!("No high scores file found at {:?}, starting fresh", path);
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(scores) => {
                    info!("Loaded high scores from {:?}", path);
                    scores
                }
                Err(e) => {
                    warn!("Failed to parse high scores: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                warn!("Failed to read high scores file: {}", e);
                Self::default()
            }
        }
    }

    /// Save high scores to disk.
    pub fn save(&self) {
        let Some(path) = Self::file_path() else {
            warn!("Could not determine data directory for saving high scores");
            return;
        };

        // Create parent directory if needed
        if let Some(parent) = path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            warn!("Failed to create high scores directory: {}", e);
            return;
        }

        match serde_json::to_string_pretty(self) {
            Ok(json) => match fs::write(&path, json) {
                Ok(()) => info!("Saved high scores to {:?}", path),
                Err(e) => warn!("Failed to write high scores: {}", e),
            },
            Err(e) => warn!("Failed to serialize high scores: {}", e),
        }
    }
}

/// Load high scores on startup.
fn load_high_scores(mut high_scores: ResMut<HighScores>) {
    *high_scores = HighScores::load();
}
