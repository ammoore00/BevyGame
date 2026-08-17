use crate::characters::npc::ai::pathfinding::pathfinder::{PathfindRequest, PathfinderState, Waypoints};
use crate::characters::npc::ai::pathfinding::{
    PathfinderQuery, PathfinderQueryItem, TARGET_REACHED_THRESHOLD,
};
use crate::characters::npc::ai::{AiStateKind, AiSystems};
use crate::debug::TileNavMap;
use bevy::prelude::*;
use common::{marker, TileCoords};
use rand::{Rng, RngExt};
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_pathfinder_wander_state.in_set(AiSystems::Calculate),
    );
}

const DEFAULT_WANDER_RANGE: u32 = 5;
const DEFAULT_MAX_IDLE_TIME: u64 = 1;
const DEFAULT_MAX_MOVEMENT_TIME: u64 = 10;

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct Wandering;

/// Stores information about what behavior the NPC should use while wandering.
/// 
/// This is a data struct and does not indicate whether the NPC is currently wandering.
#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct WanderData {
    wander_range: u32,
    max_idle_time: Duration,
    max_movement_time: Duration,
}
impl Default for WanderData {
    fn default() -> Self {
        Self {
            wander_range: DEFAULT_WANDER_RANGE,
            max_idle_time: Duration::from_secs(DEFAULT_MAX_IDLE_TIME),
            max_movement_time: Duration::from_secs(DEFAULT_MAX_MOVEMENT_TIME),
        }
    }
}

fn update_pathfinder_wander_state(
    mut pathfinder_query: Query<PathfinderQuery>,
    nav_map_query: Query<&TileNavMap>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let nav_map = nav_map_query.single();
    let Ok(nav_map) = nav_map else {
        error!("Failed to get nav map!: {:?}", nav_map.err().unwrap());
        return;
    };

    for mut pathfinder in pathfinder_query.iter_mut() {
        if pathfinder.ai_state.current == AiStateKind::Wander {
            update_wander_path(pathfinder, nav_map, *time, commands.reborrow());
        } else if pathfinder.ai_state.prev == AiStateKind::Wander {
            pathfinder.pathfinder.state = PathfinderState::Idle;
        }
    }
}

// TODO: Some of this behavior should be generic pathfinding behavior
fn update_wander_path(
    data: PathfinderQueryItem,
    nav_map: &TileNavMap,
    time: Time,
    mut commands: Commands,
) {
    let PathfinderQueryItem {
        entity,
        mut pathfinder,

        pending_task,
        waypoints,

        pos,
        collider,
        ..
    } = data;

    pathfinder.time_in_state += time.delta();

    // If we have a path, move towards it and return
    if let Some(mut waypoints) = waypoints {
        let target = waypoints.next_position;
        pathfinder.state = PathfinderState::Moving;

        // If the path is invalid, clear it
        let Some(target) = target else {
            commands.entity(entity).remove::<Waypoints>();
            pathfinder.time_in_state = Duration::ZERO;
            pathfinder.state = PathfinderState::Idle;
            error!("Invalid NPC target, stopping movement");
            return;
        };

        let distance = target.distance(*pos.0 - Vec3::Y);

        // If we are within the threshold of the next waypoint
        if distance <= TARGET_REACHED_THRESHOLD {
            // If it is the last waypoint, clear the path
            if target == waypoints.target {
                commands.entity(entity).remove::<Waypoints>();
                pathfinder.time_in_state = Duration::ZERO;
                pathfinder.state = PathfinderState::Idle;
                info!("NPC reached target! Stopping movement");
            } else {
                // Otherwise, increment the path to the next waypoint
                waypoints.increment_position();
            }
        }

        return;
    }

    // If we don't have a path, check the current pathfinder state and any pending pathfind requests
    match pathfinder.state {
        // If we are idle with no pending task and no current path,
        //  we should check against the idle timer
        //  then request a new path
        PathfinderState::Idle => {
            // If we are idle, don't have a path, but do have a queued pathfinding request, continue waiting
            if pending_task.is_some() {
                info!("NPC still searching for path");
                return;
            }

            /*
            if pathfinder.time_in_state < wander_data.max_idle_time {
                return;
            }

            pathfinder.time_in_state = Duration::ZERO;

            let tile_coords = TileCoords::from(pos.0);
            let tile_coords = *tile_coords - IVec3::Y;

            let target = select_random_wander_target(
                nav_map,
                tile_coords.into(),
                wander_data.wander_range,
                rand::rng(),
            );

            let collider_size = collider.size();
            let clearance_half_width = collider_size.x.max(collider_size.z) / 2.0;
            let clearance_height = collider_size.y;

            let request = PathfindRequest::new(tile_coords.into(), target.into(), clearance_half_width, clearance_height);
            commands.entity(entity).insert(request);

            info!("NPC started searching");
            
             */
        }
        // If we are somehow in the moving state but don't have a path, set the state to idle
        PathfinderState::Moving => {
            pathfinder.time_in_state = Duration::ZERO;
            pathfinder.state = PathfinderState::Idle;
            info!("NPC has no target, setting to idle!");
        }
        // Dispatch is handled on a per-pathfinding strategy basis, so we do nothing here
        PathfinderState::Dispatch => {}
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
