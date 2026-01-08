//! Game polish/juice effects - screen shake, pop animations, combo text.

use bevy::prelude::*;
use rand::Rng;

use super::{
    bubble::Bubble,
    cluster::{ClusterPopped, FloatingBubblesRemoved},
    hex::{GridOffset, HEX_SIZE},
    projectile::BubbleInDangerZone,
};
use crate::{screens::Screen, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    // Screen shake
    app.init_resource::<ScreenShake>();
    app.add_systems(
        Update,
        (trigger_shake_on_events, apply_screen_shake)
            .chain()
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );

    // Pop animation
    app.add_systems(
        Update,
        animate_pop
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );

    // Combo text
    app.add_systems(
        Update,
        (spawn_combo_text, animate_combo_text)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

// =============================================================================
// SCREEN SHAKE
// =============================================================================


/// Resource tracking screen shake state.
#[derive(Resource, Default)]
pub struct ScreenShake {
    /// Current trauma level (0.0 to 1.0).
    pub trauma: f32,
    /// Base position to return to.
    pub base_position: Vec3,
}

/// Maximum shake offset in pixels.
const MAX_SHAKE_OFFSET: f32 = 10.0;
/// How fast trauma decays per second.
const TRAUMA_DECAY: f32 = 2.5;


/// Trigger screen shake from game events.
fn trigger_shake_on_events(
    mut shake: ResMut<ScreenShake>,
    mut cluster_events: MessageReader<ClusterPopped>,
    mut danger_events: MessageReader<BubbleInDangerZone>,
    mut floating_events: MessageReader<FloatingBubblesRemoved>,
) {
    // Cluster popped - shake scales with size
    for event in cluster_events.read() {
        let intensity = match event.count {
            0..=3 => 0.4,
            4..=5 => 0.55,
            6..=7 => 0.7,
            _ => 0.85,
        };
        shake.trauma = (shake.trauma + intensity).min(1.0);
        info!("Screen shake from cluster: {} bubbles, trauma={}", event.count, shake.trauma);
    }

    // Danger zone - strong shake
    for _ in danger_events.read() {
        shake.trauma = 1.0;
        info!("Screen shake from danger zone!");
    }

    // Floating bubbles removed - medium shake
    for event in floating_events.read() {
        let intensity = (event.count as f32 * 0.15).min(0.6);
        shake.trauma = (shake.trauma + intensity).min(1.0);
    }
}

/// Apply screen shake to camera.
fn apply_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    if shake.trauma > 0.0 {
        let mut rng = rand::rng();

        // Shake amount = trauma^2 (makes it feel more natural)
        let shake_amount = shake.trauma * shake.trauma;

        // Random offset
        let offset_x = rng.random_range(-1.0..1.0) * MAX_SHAKE_OFFSET * shake_amount;
        let offset_y = rng.random_range(-1.0..1.0) * MAX_SHAKE_OFFSET * shake_amount;

        // Apply offset from base position
        camera_transform.translation.x = shake.base_position.x + offset_x;
        camera_transform.translation.y = shake.base_position.y + offset_y;

        // Decay trauma
        shake.trauma = (shake.trauma - TRAUMA_DECAY * time.delta_secs()).max(0.0);
    } else {
        // Reset to base position
        camera_transform.translation.x = shake.base_position.x;
        camera_transform.translation.y = shake.base_position.y;
    }
}

// =============================================================================
// POP ANIMATION
// =============================================================================

/// Component for bubbles that are popping (scale up then despawn).
#[derive(Component)]
pub struct PopAnimation {
    /// Time elapsed in the animation.
    pub timer: f32,
    /// Total animation duration.
    pub duration: f32,
    /// Starting scale.
    pub start_scale: Vec3,
    /// Target scale at peak.
    pub peak_scale: Vec3,
}

impl PopAnimation {
    pub fn new(current_scale: Vec3) -> Self {
        Self {
            timer: 0.0,
            duration: 0.15,
            start_scale: current_scale,
            peak_scale: current_scale * 1.4,
        }
    }
}

/// Animate popping bubbles and despawn when done.
fn animate_pop(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut PopAnimation)>,
) {
    for (entity, mut transform, mut pop) in &mut query {
        pop.timer += time.delta_secs();
        let progress = (pop.timer / pop.duration).min(1.0);

        // Scale up quickly, then shrink to nothing
        let scale = if progress < 0.5 {
            // First half: scale up to peak
            let t = progress * 2.0;
            pop.start_scale.lerp(pop.peak_scale, t)
        } else {
            // Second half: shrink to zero
            let t = (progress - 0.5) * 2.0;
            pop.peak_scale.lerp(Vec3::ZERO, t)
        };

        transform.scale = scale;

        // Despawn when animation complete
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// COMBO TEXT
// =============================================================================

/// Component for floating combo text.
#[derive(Component)]
pub struct ComboText {
    /// Time elapsed.
    pub timer: f32,
    /// Total duration.
    pub duration: f32,
    /// Starting position.
    pub start_y: f32,
    /// Float distance.
    pub float_distance: f32,
}

/// Spawn combo text when clusters pop.
fn spawn_combo_text(
    mut commands: Commands,
    mut cluster_events: MessageReader<ClusterPopped>,
    grid_offset: Res<GridOffset>,
    _bubble_query: Query<&Transform, With<Bubble>>,
) {
    for event in cluster_events.read() {
        // Only show combo text for clusters > 3
        if event.count <= 3 {
            continue;
        }

        // Calculate center position of the cluster
        let center_pos = if !event.coords.is_empty() {
            let sum: Vec2 = event.coords
                .iter()
                .map(|coord| coord.to_pixel_with_offset(HEX_SIZE, grid_offset.y))
                .fold(Vec2::ZERO, |acc, pos| acc + pos);
            sum / event.coords.len() as f32
        } else {
            Vec2::ZERO
        };

        // Determine text based on combo size
        let text = if event.count >= 8 {
            format!("MASSIVE! +{}!", event.count)
        } else if event.count >= 6 {
            format!("COMBO! +{}!", event.count)
        } else {
            format!("+{}!", event.count)
        };

        commands.spawn((
            Name::new("Combo Text"),
            ComboText {
                timer: 0.0,
                duration: 0.8,
                start_y: center_pos.y,
                float_distance: 50.0,
            },
            Text2d::new(text),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 0.2)),
            Transform::from_translation(center_pos.extend(10.0))
                .with_scale(Vec3::splat(0.5)),
            DespawnOnExit(Screen::Gameplay),
        ));
    }
}

/// Animate combo text (float up and fade out).
fn animate_combo_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ComboText, &mut TextColor)>,
) {
    for (entity, mut transform, mut combo, mut color) in &mut query {
        combo.timer += time.delta_secs();
        let progress = (combo.timer / combo.duration).min(1.0);

        // Scale up at start, then hold
        let scale = if progress < 0.2 {
            let t = progress / 0.2;
            0.5 + t * 1.0 // 0.5 -> 1.5
        } else {
            1.5
        };
        transform.scale = Vec3::splat(scale);

        // Float upward
        transform.translation.y = combo.start_y + combo.float_distance * progress;

        // Fade out in last 30%
        let alpha = if progress > 0.7 {
            1.0 - (progress - 0.7) / 0.3
        } else {
            1.0
        };
        color.0 = Color::srgba(1.0, 1.0, 0.2, alpha);

        // Despawn when done
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}
