use crate::characters::npc::ai::pathfinding::{
    PathfinderData, PathfinderDataItem, PathfinderSystems,
};
use crate::debug::TileNavMap;
use crate::level::LEVEL_LOADED;
use crate::level::grid::nav::NavEdgeKind;
use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use common::{TileCoords, WorldCoords};
use getset::{CopyGetters, Getters};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::fmt::Debug;
use std::ops::AddAssign;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_pathfinder_state.in_set(PathfinderSystems::Update),
            (process_pathfind_requests, collect_pathfind_requests)
                .in_set(PathfinderSystems::Collect),
        ),
    );

    app.add_observer(on_cancel_pathing.run_if(in_state(LEVEL_LOADED)));
}

pub const TARGET_REACHED_THRESHOLD: f32 = 0.2;

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, CopyGetters)]
pub struct Pathfinder {
    #[getset(get_copy = "pub")]
    state: PathfinderState,
    #[getset(get_copy = "pub")]
    time_in_state: Duration,
}
impl Pathfinder {
    pub fn set_state(&mut self, state: PathfinderState) {
        self.state = state;
        self.time_in_state = Duration::ZERO;
    }

    pub fn increment_timer<T>(&mut self, delta: T)
    where
        Duration: AddAssign<T>,
    {
        self.time_in_state.add_assign(delta);
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathfinderState {
    /// Pathfinder is idle and open to new requests
    #[default]
    Idle,
    /// Pathfinder is currently searching for a path
    Searching,
    /// Pathfinder is ready for a new path
    Dispatch,
    /// Pathfinder has a path and is moving towards it
    Moving,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Getters)]
pub struct PathfindRequest {
    #[getset(get = "pub")]
    start: WorldCoords,
    #[getset(get = "pub")]
    target: WorldCoords,
    #[getset(get = "pub")]
    clearance_half_width: f32,
    #[getset(get = "pub")]
    clearance_height: f32,

    request_id: Uuid,
}
impl PathfindRequest {
    pub fn new(
        start: WorldCoords,
        target: WorldCoords,
        clearance_half_width: f32,
        clearance_height: f32,
    ) -> Self {
        Self {
            start,
            target,
            clearance_half_width,
            clearance_height,
            request_id: Uuid::new_v4(),
        }
    }
}

#[derive(Component)]
pub struct PathfindPending {
    request: PathfindRequest,
    task: Task<Option<Waypoints>>,
    task_cancel_token: PathfindCancelToken,
}
impl PathfindPending {
    pub fn cancel(&mut self) {
        self.task_cancel_token.cancel();
    }
}
impl Debug for PathfindPending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathfindPending")
            .field("request", &self.request)
            .finish()
    }
}

/// Cancel the current path, including any pending pathfinding requests and tasks.
#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CancelPathing(pub Entity);

fn on_cancel_pathing(
    event: On<CancelPathing>,
    mut pending_pathfind_query: Query<PathfinderData>,
    mut commands: Commands,
) {
    let Ok(mut data) = pending_pathfind_query.get_mut(event.0) else {
        error!("Cannot cancel pathfinding for an entity without a pathfinder!");
        return;
    };

    data.pathfinder.set_state(PathfinderState::Idle);
    commands
        .entity(event.0)
        .remove::<(Waypoints, PathfindRequest)>();

    if let Some(mut pending) = data.pending_task {
        pending.cancel();
    }
}

#[derive(Component, Debug, Clone, Getters)]
pub struct Waypoints {
    pub(super) path: Vec<WorldCoords>,
    pub(super) target: WorldCoords,
    pub(super) next_position: Option<WorldCoords>,
    pub(super) next_index: usize,
}
impl Waypoints {
    pub(super) fn new(path: Vec<WorldCoords>) -> Self {
        let target = *path.last().unwrap();
        let next_position = *path.first().unwrap();
        Self {
            path,
            target,
            next_position: Some(next_position),
            next_index: 0,
        }
    }

    pub(super) fn increment_position(&mut self) {
        self.next_index += 1;
        self.next_position = self.path.get(self.next_index).copied();
    }

    pub fn _get_remaining_path(&self) -> &[WorldCoords] {
        &self.path[self.next_index..]
    }
}

/// Updates pathfinder state and signals for pathing dispatch
fn update_pathfinder_state(
    mut pathfinder_query: Query<PathfinderData>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for data in pathfinder_query.iter_mut() {
        let PathfinderDataItem {
            entity,
            mut pathfinder,

            pending_task,
            waypoints,

            pos,
            ..
        } = data;

        pathfinder.increment_timer(time.delta());

        // If we have a path, move towards it and return
        if let Some(mut waypoints) = waypoints {
            let target = waypoints.next_position;

            // If the path is invalid, clear it
            let Some(target) = target else {
                commands.entity(entity).remove::<Waypoints>();
                pathfinder.set_state(PathfinderState::Idle);
                error!("Invalid NPC target, stopping movement");
                return;
            };

            let distance = target.distance(*pos.0 - Vec3::Y);

            // If we are within the threshold of the next waypoint
            if distance <= TARGET_REACHED_THRESHOLD {
                // If it is the last waypoint, clear the path
                if target == waypoints.target {
                    commands.entity(entity).remove::<Waypoints>();
                    pathfinder.set_state(PathfinderState::Idle);
                    info!("NPC reached target! Stopping movement");
                } else {
                    // Otherwise, increment the path to the next waypoint
                    waypoints.increment_position();
                }
            }

            pathfinder.set_state(PathfinderState::Moving);

            return;
        }

        // If we don't have a path, check the current pathfinder state and any pending pathfind requests
        match pathfinder.state() {
            // If we are idle with no pending task and no current path,
            //  we should check against the idle timer
            //  then request a new path
            PathfinderState::Idle => {
                // If we somehow got into the idle state while searching, something went wrong
                // Log the error and correct the state
                if pending_task.is_some() {
                    error!("NPC in Idle pathfind state while executing pathfinding request!");
                    pathfinder.set_state(PathfinderState::Searching);
                    return;
                }

                pathfinder.set_state(PathfinderState::Dispatch);
            }
            // Dispatch is handled on a per-pathfinding strategy basis, so no extra logic is necessary
            // besides a check for erroneous state
            PathfinderState::Dispatch => {
                if pending_task.is_some() {
                    pathfinder.set_state(PathfinderState::Searching);
                    error!("NPC in Dispatch pathfind state while executing pathfinding request!");
                }
            }
            // If we are still waiting for a path, do nothing
            PathfinderState::Searching => {
                info!("NPC still searching!");
            }
            // If we are somehow in the moving state but don't have a path, set the state to idle
            PathfinderState::Moving => {
                pathfinder.set_state(PathfinderState::Idle);
                info!("NPC has no path, setting to idle!");
            }
        }
    }
}

/// Dispatch async tasks to find paths for pending requests.
fn process_pathfind_requests(
    requests_query: Query<(Entity, &mut Pathfinder, &PathfindRequest), Without<PathfindPending>>,
    nav_map_query: Query<&TileNavMap>,
    mut commands: Commands,
) {
    let nav_map = nav_map_query.single();
    let Ok(nav_map) = nav_map else {
        error!("Failed to get nav map!: {:?}", nav_map.err().unwrap());
        return;
    };

    let task_pool = AsyncComputeTaskPool::get();

    for (entity, mut pathfinder, request) in requests_query {
        let nav_map = nav_map.clone();
        let request = *request;

        let task_cancel_token = PathfindCancelToken::new();
        let cloned_token = task_cancel_token.clone();

        let task = task_pool.spawn(async move { find_path(&nav_map, &request, &cloned_token) });

        commands.entity(entity).insert(PathfindPending {
            request,
            task,
            task_cancel_token,
        });

        pathfinder.set_state(PathfinderState::Searching);
    }
}

// TODO: Improve cancellation logic to protect against rapid repathing locking out pathfinder
/// Process pending requests and attach a waypoint component for finished paths
fn collect_pathfind_requests(
    requests_query: Query<(Entity, &mut PathfindPending, &PathfindRequest), With<Pathfinder>>,
    mut commands: Commands,
) {
    for (entity, mut pathfind_pending, request) in requests_query {
        let request_valid = pathfind_pending.request.request_id == request.request_id;
        if !request_valid {
            pathfind_pending.task_cancel_token.cancel();
        }

        let mut entity_commands = commands.entity(entity);
        entity_commands.remove::<PathfindPending>();

        let Some(result) = block_on(poll_once(&mut pathfind_pending.task)) else {
            continue;
        };

        if request_valid {
            entity_commands.remove::<PathfindRequest>();
            if let Some(waypoints) = result {
                entity_commands.insert(waypoints);
            } else {
                info!("No path found for entity {:?}", entity);
            }
        }
    }
}

#[derive(Clone)]
pub struct PathfindCancelToken {
    cancelled: Arc<AtomicBool>,
}
impl PathfindCancelToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, AtomicOrdering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(AtomicOrdering::Relaxed)
    }
}

/// Use Theta* pathfinding to find a path from start to target
///
/// See [here](https://web.archive.org/web/20100916211209/http://aigamedev.com/open/tutorials/theta-star-any-angle-paths/)
/// or [here](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter16_Theta_Star_for_Any-Angle_Pathfinding.pdf)
/// for more information on Theta* pathfinding.
///
/// Returns `None` if a path cannot be found.
pub fn find_path(
    nav_map: &TileNavMap,
    request: &PathfindRequest,
    cancel_token: &PathfindCancelToken,
) -> Option<Waypoints> {
    // TODO: account movement capabilities, add search timeout, add LoS caching

    // Sanity check
    // The code would return the correct result anyway,
    // but this early guard prevents unnecessary computation
    if request.start == request.target {
        return Some(Waypoints::new(vec![request.start]));
    }

    let mut costs = BTreeMap::new();
    costs.insert(request.start, 0);

    let mut parents = BTreeMap::new();

    let mut heap = BinaryHeap::new();
    heap.push(PathfindCoordState {
        cost: 0,
        heuristic_cost: request.start.distance(*request.target) as u32,
        position: request.start,
    });

    // Explore frontier using min heap to explore lower cost nodes first
    while let Some(node) = heap.pop() {
        if cancel_token.is_cancelled() {
            return None;
        }

        let PathfindCoordState { cost, position, .. } = &node;
        if *position == request.target {
            let mut position = position;
            let mut path = Vec::new();

            while let Some(parent) = parents.get(position) {
                path.push(*position);
                position = parent;
            }

            if *position != request.start {
                error!("Pathfinding failed to find a path from start to target!");
                return None;
            }
            path.push(request.start);
            path.reverse();

            return Some(Waypoints::new(path));
        }

        // If we already found a better path here, skip this node
        if let Some(prev_cost) = costs.get(position)
            && cost != prev_cost
        {
            continue;
        }

        // Get all outgoing connections from the current frontier node
        let Some(edges) = nav_map.get_edges_from_tile(&position.into()) else {
            continue;
        };

        for edge in edges {
            // Try to shortcut to the grandparent based on line-of-sight
            let (next_cost, next_pos, next_parent) = if let Some(grandparent) =
                parents.get(position)
                && edge.1.kind() == NavEdgeKind::Walk
                && nav_map.has_line_of_sight(
                    &TileCoords::from(grandparent),
                    edge.0.end(),
                    request.clearance_half_width,
                    request.clearance_height,
                ) {
                let next_pos = WorldCoords::from(edge.0.end());

                // Look up how much it actually cost to get to the grandparent
                let grandparent_cost = costs.get(grandparent).copied().unwrap_or(0);

                // Calculate true distance from grandparent to the neighbor tile
                let walk_cost = edge.1.cost(); // We know this is a walk edge, so just check the cost here to avoid magic numbers
                let distance_cost = (next_pos.distance(**grandparent) * walk_cost as f32) as u32;

                (grandparent_cost + distance_cost, next_pos, grandparent)
            } else {
                (
                    cost + edge.1.cost(),
                    WorldCoords::from(edge.0.end()),
                    position,
                )
            };

            let next_heuristic_cost = next_pos.distance(*request.target) as u32;

            // If the position isn't tracked yet, default to true.
            // If it is tracked, evaluate if our new cost is cheaper
            let is_cheaper = costs
                .get(&next_pos)
                .is_none_or(|&prev_cost| next_cost < prev_cost);

            if is_cheaper {
                parents.insert(next_pos, *next_parent);
                costs.insert(next_pos, next_cost);

                heap.push(PathfindCoordState {
                    cost: next_cost,
                    heuristic_cost: next_heuristic_cost,
                    position: next_pos,
                });
            }
        }
    }

    None
}

/// Tracks the pathfinding state for a single coordinate
#[derive(Clone, PartialEq, Eq)]
struct PathfindCoordState {
    cost: u32,
    heuristic_cost: u32,
    position: WorldCoords,
}
impl Ord for PathfindCoordState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Note flipped ordering on costs to allow for min heap
        // In case of tie, we compare positions lexicographically
        // This keeps implementations of Ord and Eq consistent
        (other.cost + other.heuristic_cost)
            .cmp(&(self.cost + self.heuristic_cost))
            .then_with(|| self.position.cmp(&other.position))
    }
}
impl PartialOrd for PathfindCoordState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
