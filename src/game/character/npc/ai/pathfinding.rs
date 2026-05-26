use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::sync::Arc;
use std::time::Duration;
use bevy::prelude::*;
use rand::{Rng, RngExt};
use crate::{AppSystems, PausableSystems};
use crate::game::character::state::{try_set_state, ActionState, ActionStateTracker, ActionStateEvent};
use crate::game::character::state::action_states::{Idle, Running, Walking};
use crate::game::character::state::state_transitions::ActionStateCapabilities;
use crate::game::level::grid::coords::{TileCoords, WorldCoords, WorldPosition};
use crate::game::level::grid::nav::TileNavMap;
use crate::game::physics::movement::MovementController;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_pathfinder_wander_state,
            update_movement_intent,
            update_movement_state,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

pub(super) fn pathfinder_bundle() -> impl Bundle {
    let controller = MovementController{
        max_speed: 2.0,
        ..Default::default()
    };

    (
        Pathfinder::default(),
        RandomWander::default(),
        controller,
    )
}

#[derive(Component, Debug, Clone, Default)]
struct Pathfinder {
    state: PathfinderState,
}

#[derive(Debug, Clone, Default)]
enum PathfinderState {
    #[default]
    Idle,
    Searching,
    Moving(TargetLocation),
}

#[derive(Debug, Clone)]
enum TargetLocation {
    Wander(WorldCoords),
    Target(WorldCoords),
}
impl TargetLocation {
    fn get(&self) -> &WorldCoords {
        match self {
            TargetLocation::Wander(coords)
            | TargetLocation::Target(coords) => coords
        }
    }
}

struct TilePath {
    path: Vec<WorldCoords>,
    target: WorldCoords,
    next_index: usize,
}
impl TilePath {
    fn new(path: Vec<WorldCoords>) -> Self {
        let target = path.last().unwrap().clone();
        Self {
            path,
            target,
            next_index: 0,
        }
    }
}

const DEFAULT_WANDER_RANGE: u32 = 5;
const DEFAULT_MAX_IDLE_TIME: u64 = 1;
const DEFAULT_MAX_MOVEMENT_TIME: u64 = 10;

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct RandomWander {
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
    pathfinder_query: Query<(&mut Pathfinder, &mut RandomWander, &WorldPosition)>,
    nav_map_query: Query<&TileNavMap>,
) {
    let nav_map = nav_map_query.single();
    let Ok(nav_map) = nav_map else {
        error!("Failed to get nav map!: {:?}", nav_map.err().unwrap());
        return;
    };

    for (
        mut pathfinder,
        mut wander,
        pos
    ) in pathfinder_query {
        wander.current_time_in_state += time.delta();

        match &pathfinder.state {
            PathfinderState::Idle => {
                if wander.current_time_in_state >= wander.max_idle_time {
                    wander.current_time_in_state = Duration::ZERO;
                    pathfinder.state = PathfinderState::Searching;
                    info!("NPC started searching");
                }
            }
            PathfinderState::Searching => {
                let tile_coords = TileCoords::from(pos.0.clone());
                let tile_coords = *tile_coords - IVec3::Y;

                if let Some(tile_path) = select_random_wander_target_old(
                    nav_map,
                    &tile_coords.into(),
                    wander.wander_range,
                    rand::rng()
                ) {
                    wander.current_time_in_state = Duration::ZERO;

                    let target = tile_path.path.last().unwrap().clone();
                    let target = TargetLocation::Wander(target.into());

                    info!("NPC found target: {:?}, starting movement", target.clone());
                    pathfinder.state = PathfinderState::Moving(target);
                }
            }
            PathfinderState::Moving(target) => {
                let target = target.get();
                let distance = target.distance(*pos.0 - Vec3::Y);

                if distance <= TARGET_REACHED_THRESHOLD {
                    wander.current_time_in_state = Duration::ZERO;
                    pathfinder.state = PathfinderState::Idle;
                    info!("NPC reached target! Stopping movement");
                }
            }
        }
    }
}

fn update_movement_intent(
    pathfinder_query: Query<(&Pathfinder, &mut MovementController, &WorldPosition)>,
) {
    for (
        pathfinder,
        mut controller,
        pos
    ) in pathfinder_query {
        if let PathfinderState::Moving(target) = &pathfinder.state {
            let delta = **target.get() - *pos.0;
            let delta = delta * Vec3::new(1., 0., 1.);

            controller.intent = if delta.length() > 1. {
                delta.normalize()
            } else {
                delta
            };
        } else {
            controller.intent = Vec3::ZERO;
        }
    }
}

fn update_movement_state(
    world: &mut World
) {
    let mut npc_query = world.query_filtered::<Entity, (
        With<MovementController>,
        With<ActionStateTracker>,
        With<Pathfinder>,
        With<ActionStateCapabilities>,
    )>();
    let npc_query: Vec<_> = npc_query.iter(world).collect();

    for entity in npc_query {
        let controller = world.get::<MovementController>(entity).unwrap();

        let new_state: Box<dyn ActionState> = if controller.intent.length() > 0.7 {
            Box::new(Running)
        } else if controller.intent.length() > 0.01 {
            Box::new(Walking)
        } else {
            Box::new(Idle)
        };

        if let Err(err) = try_set_state(entity, new_state, world) {
            error!("Failed to set movement state for NPC: {}", err);
            continue;
        }
    }
}

/// Selects a random wander target starting from the given coordinates.
///
/// Uses a breadth-first search to guarantee that the target is reachable.
///
/// Parameters:
/// - `start`: The starting tile coordinates to begin searching
/// - `range`: The maximum distance to search for a wander target.
///   Note that this is the actual distance traveled, not the geometric distance,
///   and is measured as taxicab distance
///
/// Returns `None` if no valid wander target could be found
fn select_random_wander_target_old(
    nav_map: &TileNavMap,
    start: &TileCoords,
    _range: u32,
    mut rand: impl Rng,
) -> Option<TilePath> {
    // TODO: Implement actual ranged pathfinding instead of only simple adjacency
    if !nav_map.has_node(start.clone()) {
        error!("Invalid start position for wander target selection: {:?}", start);
        return None;
    }

    let edges = nav_map.get_edges_from_tile(start.clone())?;
    let idx = rand.random_range(..edges.len());

    let edge = edges[idx].0;

    let path = vec![edge.end().clone().into()];
    Some(TilePath::new(path))
}

/// Selects a random wander target starting from the given coordinates using a random walk
///
/// This guarantees that the target is pathable. Even though the walk length is fixed, the actual
/// distance of the target from the start is random, bounded by the length.
fn select_random_wander_target(
    nav_map: &TileNavMap,
    start: &TileCoords,
    distance: u32,
    mut rand: impl Rng,
) -> TileCoords {
    if !nav_map.has_node(start.clone()) {
        error!("Invalid start position for wander target selection: {:?}", start);
        return start.clone();
    }

    // Vec instead of HashSet since collection sizes will always be small
    let mut visited = vec![start];

    let mut target = start;
    for _ in 0..distance {
        let Some(edges) = nav_map.get_edges_from_tile(target.clone()) else {
            error!("No valid edges found for wander target selection from {:?}", target);
            continue;
        };

        let edges = edges.iter()
            .filter(|edge| !visited.contains(&edge.0.end()))
            .collect::<Vec<_>>();

        // If we've backed ourselves into a corner, just return early
        if edges.is_empty() {
            return target.clone();
        }

        let idx = rand.random_range(..edges.len());
        target = edges[idx].0.end();
        visited.push(target);
    }
    target.clone()
}

/// Use Theta* pathfinding to find a path from start to target
///
/// See [here](https://web.archive.org/web/20100916211209/http://aigamedev.com/open/tutorials/theta-star-any-angle-paths/)
/// or [here](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter16_Theta_Star_for_Any-Angle_Pathfinding.pdf)
/// for more information on Theta* pathfinding.
///
/// Returns `None` if a path cannot be found.
fn find_path(
    nav_map: &TileNavMap,
    start: &WorldCoords,
    target: &WorldCoords,
) -> Option<TilePath> {
    // TODO: account for clearance and movement capabilities, add search timeout
    // TODO: Make this actually use Theta*, as right now this is just basic A*

    // Sanity check
    // The code would return the correct result anyway,
    // but this early guard prevents unnecessary computation
    if start == target {
        return Some(TilePath::new(vec![start.clone()]));
    }

    let mut costs = BTreeMap::new();
    costs.insert(start.clone(), 0);

    let mut parents = BTreeMap::new();

    let mut heap = BinaryHeap::new();
    heap.push(PathfindCoordState {
        cost: 0,
        position: start.clone(),
    });

    // Explore frontier using min heap to explore lower cost nodes first
    while let Some(node) = heap.pop() {
        let PathfindCoordState { cost, position } = &node;
        if position == target {
            let mut position = position;
            let mut path = Vec::new();

            while let Some(parent) = parents.get(position) {
                path.push(position.clone());
                position = parent;
            }

            if position != start {
                error!("Pathfinding failed to find a path from start to target!");
                return None;
            }
            path.push(start.clone());
            path.reverse();

            return Some(TilePath::new(path));
        }

        // If we already found a better path here, skip this node
        if let Some(prev_cost) = costs.get(position)
            && cost > prev_cost
        {
            continue;
        }

        // Get all outgoing connections from the current frontier node
        let Some(edges) = nav_map.get_edges_from_tile(position.into()) else {
            continue;
        };

        for edge in edges {
            let next_cost = cost + edge.1.cost();
            let next_pos: WorldCoords = edge.0.end().into();

            // If the position isn't tracked yet, default to true.
            // If it is tracked, evaluate if our new cost is cheaper.
            let is_cheaper = costs.get(&next_pos)
                .is_none_or(|&prev_cost| next_cost < prev_cost);

            if is_cheaper {
                parents.insert(next_pos.clone(), position.clone());
                costs.insert(next_pos.clone(), next_cost);

                heap.push(PathfindCoordState {
                    cost: next_cost,
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
    position: WorldCoords,
}
impl Ord for PathfindCoordState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Note flipped ordering on costs to allow for min heap
        // In case of tie, we compare positions lexicographically
        // This keeps implementations of Ord and Eq consistent
        other.cost.cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}
impl PartialOrd for PathfindCoordState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}