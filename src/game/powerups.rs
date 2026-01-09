//! Power-up system - unlockable abilities earned every 5 levels.
//!
//! Power-ups are selected from a random choice of 3 at each milestone.
//! They reset each game (roguelike-style progression).

use bevy::prelude::*;
use rand::seq::SliceRandom;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UnlockedPowerUps>();
    app.init_resource::<PowerUpChoices>();
    app.register_type::<UnlockedPowerUps>();
}

/// All available power-ups.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum PowerUp {
    // Tier 1 (Levels 5, 10)
    SpeedySnord,
    EagleEye,
    LuckySnord,
    BouncySnord,
    // Tier 2 (Levels 15, 20+)
    Procrastisnord,
    FortuneSnord,
    ComboSnord,
    Sharpshooter,
}

impl PowerUp {
    /// Get the tier of this power-up (1 or 2).
    #[allow(dead_code)]
    pub fn tier(&self) -> u32 {
        match self {
            PowerUp::SpeedySnord
            | PowerUp::EagleEye
            | PowerUp::LuckySnord
            | PowerUp::BouncySnord => 1,
            PowerUp::Procrastisnord
            | PowerUp::FortuneSnord
            | PowerUp::ComboSnord
            | PowerUp::Sharpshooter => 2,
        }
    }

    /// Get the display name.
    pub fn name(&self) -> &'static str {
        match self {
            PowerUp::SpeedySnord => "Speedy Snord",
            PowerUp::EagleEye => "Eagle Eye",
            PowerUp::LuckySnord => "Lucky Snord",
            PowerUp::BouncySnord => "Bouncy Snord",
            PowerUp::Procrastisnord => "Procrastisnord",
            PowerUp::FortuneSnord => "Fortune Snord",
            PowerUp::ComboSnord => "Combo Snord",
            PowerUp::Sharpshooter => "Sharpshooter",
        }
    }

    /// Get the description.
    pub fn description(&self) -> &'static str {
        match self {
            PowerUp::SpeedySnord => "25% faster projectiles",
            PowerUp::EagleEye => "2x longer aim line",
            PowerUp::LuckySnord => "Better color matching",
            PowerUp::BouncySnord => "Shows bounce trajectory",
            PowerUp::Procrastisnord => "+2 shots before descent",
            PowerUp::FortuneSnord => "See 3 upcoming snords",
            PowerUp::ComboSnord => "+50% score for big combos",
            PowerUp::Sharpshooter => "More precise shots",
        }
    }

    /// Get all power-ups for a given tier.
    pub fn for_tier(tier: u32) -> Vec<PowerUp> {
        match tier {
            1 => vec![
                PowerUp::SpeedySnord,
                PowerUp::EagleEye,
                PowerUp::LuckySnord,
                PowerUp::BouncySnord,
            ],
            _ => vec![
                PowerUp::Procrastisnord,
                PowerUp::FortuneSnord,
                PowerUp::ComboSnord,
                PowerUp::Sharpshooter,
            ],
        }
    }

    /// Get the tier for a given level.
    pub fn tier_for_level(level: u32) -> u32 {
        if level < 15 {
            1
        } else {
            2
        }
    }

    /// Get 3 random power-ups for selection, excluding already unlocked ones.
    pub fn random_choices(level: u32, unlocked: &[PowerUp]) -> Vec<PowerUp> {
        let tier = Self::tier_for_level(level);
        let mut available: Vec<PowerUp> = Self::for_tier(tier)
            .into_iter()
            .filter(|p| !unlocked.contains(p))
            .collect();

        // If not enough in current tier, add from other tier
        if available.len() < 3 {
            let other_tier = if tier == 1 { 2 } else { 1 };
            let other: Vec<PowerUp> = Self::for_tier(other_tier)
                .into_iter()
                .filter(|p| !unlocked.contains(p))
                .collect();
            available.extend(other);
        }

        // Shuffle and take 3
        let mut rng = rand::rng();
        available.shuffle(&mut rng);
        available.into_iter().take(3).collect()
    }
}

/// Resource tracking player's unlocked power-ups (reset each game).
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct UnlockedPowerUps {
    pub powers: Vec<PowerUp>,
}

impl UnlockedPowerUps {
    /// Check if a power-up is unlocked.
    pub fn has(&self, power: PowerUp) -> bool {
        self.powers.contains(&power)
    }

    /// Add a power-up.
    pub fn add(&mut self, power: PowerUp) {
        if !self.has(power) {
            self.powers.push(power);
            info!("Power-up unlocked: {}", power.name());
        }
    }

    /// Reset all power-ups (called on game start).
    pub fn reset(&mut self) {
        self.powers.clear();
        info!("Power-ups reset");
    }
}

/// Resource holding the current power-up choices for selection.
#[derive(Resource, Default)]
pub struct PowerUpChoices {
    pub choices: Vec<PowerUp>,
    pub level: u32,
}
