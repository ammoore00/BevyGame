use crate::characters::npc::ai::pathfinding::pathfinder::{PathfindRequest, PathfinderState};
use crate::characters::npc::ai::pathfinding::strategy::{
    PathfindStrategy, PathfindStrategyRegistry, ReflectPathfindStrategy,
};
use crate::characters::npc::ai::pathfinding::{PathfinderData, PathfinderSystems};
use crate::debug::TileNavMap;
use bevy::prelude::*;
use common::TileCoords;
use rand::{Rng, RngExt};
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, wander_dispatch.in_set(PathfinderSystems::Dispatch));

    app.register_pathfind_strategy::<Wandering>();
}

const DEFAULT_WANDER_RANGE: u32 = 5;
const DEFAULT_MAX_IDLE_TIME: u64 = 1;
const DEFAULT_MAX_MOVEMENT_TIME: u64 = 10;

#[derive(Component, Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
#[reflect(PathfindStrategy)]
pub struct Wandering;
impl PathfindStrategy for Wandering {}

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

fn wander_dispatch(
    pathfinder_query: Query<(PathfinderData, &WanderData), With<Wandering>>,
    nav_map_query: Query<&TileNavMap>,
    mut commands: Commands,
) {
    let nav_map = nav_map_query.single();
    let Ok(nav_map) = nav_map else {
        error!("Failed to get nav map!: {:?}", nav_map.err().unwrap());
        return;
    };

    for (pathfinder_data, wander_data) in pathfinder_query {
        if pathfinder_data.pathfinder.state() != PathfinderState::Dispatch {
            continue;
        }

        if pathfinder_data.pathfinder.time_in_state() < wander_data.max_idle_time {
            continue;
        }

        let tile_coords = TileCoords::from(pathfinder_data.pos.0);
        let tile_coords = *tile_coords - IVec3::Y;

        let target = select_random_wander_target(
            nav_map,
            tile_coords.into(),
            wander_data.wander_range,
            rand::rng(),
        );

        let collider_size = pathfinder_data.collider.size();
        let clearance_half_width = collider_size.x.max(collider_size.z) / 2.0;
        let clearance_height = collider_size.y;

        let request = PathfindRequest::new(
            tile_coords.into(),
            target.into(),
            clearance_half_width,
            clearance_height,
        );
        commands.entity(pathfinder_data.entity).insert(request);

        info!("NPC started searching");
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
