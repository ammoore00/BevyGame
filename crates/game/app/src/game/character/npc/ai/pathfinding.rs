use crate::game::character::state::capabilities::ActionStateCapabilities;
use crate::game::character::state::states::{Idle, Running, Walking};
use crate::game::character::state::tracking::{try_set_state, ActionState, ActionStateTracker};
use crate::game::level::grid::nav::{NavEdgeKind, TileNavMap};
use crate::game::level::LevelSpawnState;
use crate::screens::Screen;
use bevy::prelude::*;
use common::{AppSystems, PausableSystems, TileCoords, WorldCoords, WorldPosition};
use getset::Getters;
use rand::{Rng, RngExt};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::time::Duration;
use physics::{Collider, MovementController};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_pathfinder_wander_state,
            update_movement_intent,
            update_movement_state,
        )
            .chain()
            .run_if(
                in_state(Screen::Gameplay)
                    .and_then(in_state(LevelSpawnState::Finished))
            )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

pub(super) fn pathfinder_scene() -> impl Scene {
    bsn! [
        Pathfinder
        RandomWander
    ]
}

#[derive(Component, Debug, Clone, Default, Getters)]
pub struct Pathfinder {
    #[getset(get = "pub")]
    state: PathfinderState,
}

#[derive(Debug, Clone, Default)]
pub enum PathfinderState {
    #[default]
    Idle,
    Searching,
    Moving(PathType),
}

#[derive(Debug, Clone)]
pub enum PathType {
    Wander(TilePath),
    _Target(TilePath),
}
impl PathType {
    pub fn get(&self) -> &TilePath {
        match self {
            PathType::Wander(path)
            | PathType::_Target(path) => path
        }
    }

    fn get_mut(&mut self) -> &mut TilePath {
        match self {
            PathType::Wander(path)
            | PathType::_Target(path) => path
        }
    }
}

#[derive(Debug, Clone, Getters)]
pub struct TilePath {
    path: Vec<WorldCoords>,
    target: WorldCoords,
    next_position: Option<WorldCoords>,
    next_index: usize,
}
impl TilePath {
    fn new(path: Vec<WorldCoords>) -> Self {
        let target = path.last().unwrap().clone();
        let next_position = path.first().unwrap().clone();
        Self {
            path,
            target,
            next_position: Some(next_position),
            next_index: 0,
        }
    }

    fn increment_position(&mut self) {
        self.next_index += 1;
        self.next_position = self.path.get(self.next_index).cloned();
    }

    pub fn get_remaining_path(&self) -> &[WorldCoords] {
        &self.path[self.next_index..]
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
    pathfinder_query: Query<(&mut Pathfinder, &mut RandomWander, &WorldPosition, &Collider)>,
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
        pos,
        collider,
    ) in pathfinder_query {
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
                let tile_coords = TileCoords::from(pos.0.clone());
                let tile_coords = *tile_coords - IVec3::Y;

                let target = select_random_wander_target(
                    nav_map,
                    &tile_coords.into(),
                    wander.wander_range,
                    rand::rng()
                );

                let collider_size = collider.size();
                let clearance_half_width = collider_size.x.max(collider_size.z) / 2.0;
                let clearance_height = collider_size.y;

                if let Some(tile_path) = find_path(
                    nav_map,
                    &tile_coords.into(),
                    &target.into(),
                    clearance_half_width,
                    clearance_height,
                ) {
                    wander.current_time_in_state = Duration::ZERO;
                    let tile_path = PathType::Wander(tile_path);

                    info!("NPC found target: {:?}, starting movement", tile_path.get().next_position.clone());
                    pathfinder.state = PathfinderState::Moving(tile_path);
                }
            }
            PathfinderState::Moving(tile_path) => {
                let tile_path = tile_path.get_mut();
                let target = tile_path.next_position.clone();

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

/// Update the movement controller based on what the pathfinder wants
fn update_movement_intent(
    pathfinder_query: Query<(&Pathfinder, &mut MovementController, &WorldPosition)>,
) {
    for (
        pathfinder,
        mut controller,
        pos
    ) in pathfinder_query {
        if let PathfinderState::Moving(target) = &pathfinder.state {
            let delta = **target.get().next_position.as_ref().unwrap() - *pos.0;
            let delta = delta * Vec3::new(1., 0., 1.);

            if delta.length() < 0.01 {
                controller.intent = Vec3::ZERO;
            } else {
                controller.intent = delta.normalize();
            }
        } else {
            controller.intent = Vec3::ZERO;
        }
    }
}

/// Update the action state based on the movement intent
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

        let state = world.get::<ActionStateTracker>(entity).unwrap();
        if (*new_state).type_id() == state.type_id {
            continue;
        }

        if let Err(err) = try_set_state(entity, new_state, world) {
            error!("Failed to set movement state for NPC: {}", err);
            continue;
        }
    }
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
    if !nav_map.has_node(start) {
        error!("Invalid start position for wander target selection: {:?}", start);
        return start.clone();
    }

    // Vec instead of HashSet since collection sizes will always be small
    let mut visited = vec![start];

    let mut target = start;
    for _ in 0..distance {
        let Some(edges) = nav_map.get_edges_from_tile(target) else {
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
    clearance_half_width: f32,
    clearance_height: f32,
) -> Option<TilePath> {
    // TODO: account movement capabilities, add search timeout

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
        heuristic_cost: start.distance(**target) as u32,
        position: start.clone(),
    });

    // Explore frontier using min heap to explore lower cost nodes first
    while let Some(node) = heap.pop() {
        let PathfindCoordState { cost, position, .. } = &node;
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
            let (
                next_cost,
                next_pos,
                next_parent
            ) = if let Some(grandparent) = parents.get(position)
                && edge.1.kind() == NavEdgeKind::Walk
                && nav_map.has_line_of_sight(
                    &TileCoords::from(grandparent),
                    edge.0.end(),
                    clearance_half_width,
                    clearance_height,
                )
            {
                let next_pos = WorldCoords::from(edge.0.end());

                // Look up how much it actually cost to get to the grandparent
                let grandparent_cost = costs.get(grandparent).copied().unwrap_or(0);

                // Calculate true distance from grandparent to the neighbor tile
                let walk_cost = edge.1.cost(); // We know this is a walk edge, so just check the cost here to avoid magic numbers
                let distance_cost = (next_pos.distance(**grandparent) * walk_cost as f32) as u32;

                (
                    grandparent_cost + distance_cost,
                    next_pos,
                    grandparent,
                )
            } else {
                (
                    cost + edge.1.cost(),
                    WorldCoords::from(edge.0.end()),
                    position,
                )
            };

            let next_heuristic_cost = next_pos.distance(**target) as u32;

            // If the position isn't tracked yet, default to true.
            // If it is tracked, evaluate if our new cost is cheaper
            let is_cheaper = costs.get(&next_pos)
                .is_none_or(|&prev_cost| next_cost < prev_cost);

            if is_cheaper {
                parents.insert(next_pos.clone(), next_parent.clone());
                costs.insert(next_pos.clone(), next_cost);

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