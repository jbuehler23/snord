//! Cluster detection - finding and popping matching bubbles.
//!
//! Uses flood fill (BFS) to find connected groups of same-colored bubbles.
//! When a cluster of 3+ is found, they pop!

use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

use super::{
    bubble::{Bubble, BubbleColor},
    grid::HexGrid,
    hex::HexCoord,
    projectile::BubbleLanded,
};
use crate::{screens::Screen, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_message::<ClusterPopped>();
    app.add_message::<FloatingBubblesRemoved>();

    // Configure system sets for proper ordering with command application between them
    app.configure_sets(
        Update,
        ClusterSystems.after(super::projectile::ProjectileSystems),
    );

    // Add ApplyDeferred to ensure bubble spawn commands are processed before cluster detection
    app.add_systems(
        Update,
        ApplyDeferred
            .after(super::projectile::ProjectileSystems)
            .before(ClusterSystems)
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_systems(
        Update,
        (
            detect_clusters,
            detect_floating_bubbles,
        )
            .chain()
            .in_set(PausableSystems)
            .in_set(ClusterSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// System set for cluster detection systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClusterSystems;

/// Minimum cluster size to pop (match-3).
const MIN_CLUSTER_SIZE: usize = 3;

/// Message sent when a cluster is popped.
#[derive(Message, Debug, Clone)]
pub struct ClusterPopped {
    pub coords: Vec<HexCoord>,
    pub color: BubbleColor,
    pub count: usize,
}

/// Message sent when floating bubbles are removed.
#[derive(Message, Debug, Clone)]
pub struct FloatingBubblesRemoved {
    pub coords: Vec<HexCoord>,
    pub count: usize,
}

/// Detect and pop clusters when a bubble lands.
fn detect_clusters(
    mut commands: Commands,
    mut grid: ResMut<HexGrid>,
    bubble_query: Query<&Bubble>,
    mut landed_events: MessageReader<BubbleLanded>,
    mut popped_events: MessageWriter<ClusterPopped>,
) {
    for event in landed_events.read() {
        // Find the cluster starting from the landed bubble
        let cluster = find_cluster(&grid, &bubble_query, event.coord, event.color);

        if cluster.len() >= MIN_CLUSTER_SIZE {
            info!(
                "Found cluster of {} {:?} bubbles at {:?}",
                cluster.len(),
                event.color,
                event.coord
            );

            // Remove all bubbles in the cluster
            for &coord in &cluster {
                if let Some(entity) = grid.remove(coord) {
                    commands.entity(entity).despawn();
                }
            }

            popped_events.write(ClusterPopped {
                coords: cluster.clone(),
                color: event.color,
                count: cluster.len(),
            });
        }
    }
}

/// Find all connected bubbles of the same color using flood fill (BFS).
///
/// Note: The start coordinate is always included in the cluster because we know
/// its color from the BubbleLanded event. This bypasses Bevy's deferred commands
/// timing issue where the newly spawned bubble's Bubble component may not exist
/// yet when we query it.
fn find_cluster(
    grid: &HexGrid,
    bubble_query: &Query<&Bubble>,
    start: HexCoord,
    target_color: BubbleColor,
) -> Vec<HexCoord> {
    let mut cluster = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // Always add the starting position - we know its color from the event
    // (bypasses deferred commands timing issue)
    cluster.push(start);
    visited.insert(start);

    // Start exploring from the starting position's neighbors
    for neighbor in start.neighbors() {
        if !visited.contains(&neighbor) {
            visited.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    // Continue BFS for neighbors
    while let Some(coord) = queue.pop_front() {
        // Check if this cell has a bubble of the right color
        if let Some(entity) = grid.get(coord) {
            if let Ok(bubble) = bubble_query.get(entity) {
                if bubble.color == target_color {
                    cluster.push(coord);

                    // Add unvisited neighbors to the queue
                    for neighbor in coord.neighbors() {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }
    }

    cluster
}

/// Detect and remove floating bubbles (not connected to top row).
fn detect_floating_bubbles(
    mut commands: Commands,
    mut grid: ResMut<HexGrid>,
    mut popped_events: MessageReader<ClusterPopped>,
    mut floating_events: MessageWriter<FloatingBubblesRemoved>,
) {
    // Only run after a cluster is popped
    let mut should_check = false;
    for _ in popped_events.read() {
        should_check = true;
    }

    if !should_check {
        return;
    }

    // Find all bubbles connected to the top row
    let anchored = find_anchored_bubbles(&grid);

    // Find floating bubbles (in grid but not anchored)
    let mut floating = Vec::new();
    for coord in grid.coords().collect::<Vec<_>>() {
        if !anchored.contains(&coord) {
            floating.push(coord);
        }
    }

    if !floating.is_empty() {
        info!("Found {} floating bubbles to remove", floating.len());

        // Remove floating bubbles
        for &coord in &floating {
            if let Some(entity) = grid.remove(coord) {
                commands.entity(entity).despawn();
            }
        }

        floating_events.write(FloatingBubblesRemoved {
            coords: floating.clone(),
            count: floating.len(),
        });
    }
}

/// Find all bubbles connected to the top row using BFS.
fn find_anchored_bubbles(grid: &HexGrid) -> HashSet<HexCoord> {
    let mut anchored = HashSet::new();
    let mut queue = VecDeque::new();

    // Start from all bubbles in the top row (r = 0)
    for coord in grid.top_row_coords() {
        queue.push_back(coord);
        anchored.insert(coord);
    }

    // BFS to find all connected bubbles
    while let Some(coord) = queue.pop_front() {
        for neighbor in coord.neighbors() {
            if !anchored.contains(&neighbor) && grid.is_occupied(neighbor) {
                anchored.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    anchored
}
