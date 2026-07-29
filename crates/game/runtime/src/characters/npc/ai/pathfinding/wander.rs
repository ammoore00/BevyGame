use crate::characters::npc::ai::pathfinding::PathfindingSystems;
use crate::characters::npc::ai::pathfinding::path::{
    PathType, Pathfinder, PathfinderState, find_path,
};
use crate::debug::TileNavMap;
use bevy::prelude::*;
use common::{TileCoords, WorldPosition};
use physics::Collider;
use rand::{Rng, RngExt};
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_pathfinder_wander_state,).in_set(PathfindingSystems::Find),
    );
}

const DEFAULT_WANDER_RANGE: u32 = 5;
const DEFAULT_MAX_IDLE_TIME: u64 = 1;
const DEFAULT_MAX_MOVEMENT_TIME: u64 = 10;

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RandomWander {
    wander_range: u32,
    max_idle_time: Duration,
    max_movement_time: Duration,
    current_time_in_state: Duration,
}
impl Default for RandomWander {
    fn default() -> Self {
        Self {
            wander_range: DEFAULT_WANDER_RANGE,
            max_idle_time: Duration::from_secs(DEFAULT_MAX_IDLE_TIME),
            max_movement_time: Duration::from_secs(DEFAULT_MAX_MOVEMENT_TIME),
            current_time_in_state: Duration::ZERO,
        }
    }
}

const TARGET_REACHED_THRESHOLD: f32 = 0.2;

fn update_pathfinder_wander_state(
    time: Res<Time>,
    pathfinder_query: Query<(
        &mut Pathfinder,
        &mut RandomWander,
        &WorldPosition,
        &Collider,
    )>,
    nav_map_query: Query<&TileNavMap>,
) {
    let nav_map = nav_map_query.single();
    let Ok(nav_map) = nav_map else {
        error!("Failed to get nav level!: {:?}", nav_map.err().unwrap());
        return;
    };

    for (mut pathfinder, mut wander, pos, collider) in pathfinder_query {
        wander.current_time_in_state += time.delta();

        match &mut pathfinder.state {
            PathfinderState::Idle => {
                if wander.current_time_in_state >= wander.max_idle_time {
                    wander.current_time_in_state = Duration::ZERO;
                    pathfinder.state = PathfinderState::Searching;
                    info!("NPC started searching");
                }
            }
            PathfinderState::Searching => {
                let tile_coords = TileCoords::from(pos.0);
                let tile_coords = *tile_coords - IVec3::Y;

                let target = select_random_wander_target(
                    nav_map,
                    tile_coords.into(),
                    wander.wander_range,
                    rand::rng(),
                );

                let collider_size = collider.size();
                let clearance_half_width = collider_size.x.max(collider_size.z) / 2.0;
                let clearance_height = collider_size.y;

                if let Some(tile_path) = find_path(
                    nav_map,
                    tile_coords.into(),
                    target.into(),
                    clearance_half_width,
                    clearance_height,
                ) {
                    wander.current_time_in_state = Duration::ZERO;
                    let tile_path = PathType::Wander(tile_path);

                    info!(
                        "NPC found target: {:?}, starting movement",
                        tile_path.get().next_position
                    );
                    pathfinder.state = PathfinderState::Moving(tile_path);
                }
            }
            PathfinderState::Moving(tile_path) => {
                let tile_path = tile_path.get_mut();
                let target = tile_path.next_position;

                let Some(target) = target else {
                    error!("Invaliud NPC target, stopping movement");
                    pathfinder.state = PathfinderState::Idle;
                    continue;
                };

                let distance = target.distance(*pos.0 - Vec3::Y);

                if distance <= TARGET_REACHED_THRESHOLD {
                    if target == tile_path.target {
                        wander.current_time_in_state = Duration::ZERO;
                        pathfinder.state = PathfinderState::Idle;
                        info!("NPC reached target! Stopping movement");
                    } else {
                        tile_path.increment_position();
                    }
                }
            }
        }
    }
}

/// Selects a random wander target starting from the given coordinates using a random walk
///
/// This guarantees that the target is pathable. Even though the walk length is fixed, the actual
/// distance of the target from the start is random, bounded by the length.
fn select_random_wander_target(
    nav_map: &TileNavMap,
    start: TileCoords,
    distance: u32,
    mut rand: impl Rng,
) -> TileCoords {
    if !nav_map.has_node(&start) {
        error!(
            "Invalid start position for wander target selection: {:?}",
            start
        );
        return start;
    }

    // Vec instead of HashSet since collection sizes will always be small
    let mut visited = vec![start];

    let mut target = start;
    for _ in 0..distance {
        let Some(edges) = nav_map.get_edges_from_tile(&target) else {
            error!(
                "No valid edges found for wander target selection from {:?}",
                target
            );
            continue;
        };

        let edges = edges
            .iter()
            .filter(|edge| !visited.contains(edge.0.end()))
            .collect::<Vec<_>>();

        // If we've backed ourselves into a corner, just return early
        if edges.is_empty() {
            return target;
        }

        let idx = rand.random_range(..edges.len());
        target = *edges[idx].0.end();
        visited.push(target);
    }
    target
}
