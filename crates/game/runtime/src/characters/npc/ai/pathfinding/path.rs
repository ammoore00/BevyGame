use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use bevy::prelude::*;
use getset::Getters;
use assets::action_states::{ActionState, ActionStateCapabilities, Idle, Running, Walking};
use common::{TileCoords, WorldCoords, WorldPosition};
use physics::MovementController;
use crate::characters::npc::ai::pathfinding::PathfindingSystems;
use crate::characters::state::{ActionStateTracker, TrySetStateEvent};
use crate::debug::TileNavMap;
use crate::level::grid::nav::NavEdgeKind;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_movement_intent, update_movement_state)
            .chain()
            .in_set(PathfindingSystems::Execute),
    );
}

#[derive(Component, Debug, Clone, Default, Getters)]
pub struct Pathfinder {
    #[getset(get = "pub")]
    pub(super) state: PathfinderState,
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
            PathType::Wander(path) | PathType::_Target(path) => path,
        }
    }

    pub(super) fn get_mut(&mut self) -> &mut TilePath {
        match self {
            PathType::Wander(path) | PathType::_Target(path) => path,
        }
    }
}

#[derive(Debug, Clone, Getters)]
pub struct TilePath {
    pub(super) path: Vec<WorldCoords>,
    pub(super) target: WorldCoords,
    pub(super) next_position: Option<WorldCoords>,
    pub(super) next_index: usize,
}
impl TilePath {
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

/// Update the movement controller based on what the pathfinder wants
fn update_movement_intent(
    pathfinder_query: Query<(&Pathfinder, &mut MovementController, &WorldPosition)>,
) {
    for (pathfinder, mut controller, pos) in pathfinder_query {
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
// TODO: Remove the direct world access here
fn update_movement_state(world: &mut World) {
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
        if (*new_state).type_id() == state.state_type_id() {
            continue;
        }

        world.trigger(TrySetStateEvent::new(entity, new_state));
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
    start: WorldCoords,
    target: WorldCoords,
    clearance_half_width: f32,
    clearance_height: f32,
) -> Option<TilePath> {
    // TODO: account movement capabilities, add search timeout

    // Sanity check
    // The code would return the correct result anyway,
    // but this early guard prevents unnecessary computation
    if start == target {
        return Some(TilePath::new(vec![start]));
    }

    let mut costs = BTreeMap::new();
    costs.insert(start, 0);

    let mut parents = BTreeMap::new();

    let mut heap = BinaryHeap::new();
    heap.push(PathfindCoordState {
        cost: 0,
        heuristic_cost: start.distance(*target) as u32,
        position: start,
    });

    // Explore frontier using min heap to explore lower cost nodes first
    while let Some(node) = heap.pop() {
        let PathfindCoordState { cost, position, .. } = &node;
        if *position == target {
            let mut position = position;
            let mut path = Vec::new();

            while let Some(parent) = parents.get(position) {
                path.push(*position);
                position = parent;
            }

            if *position != start {
                error!("Pathfinding failed to find a path from start to target!");
                return None;
            }
            path.push(start);
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
            let (next_cost, next_pos, next_parent) = if let Some(grandparent) =
                parents.get(position)
                && edge.1.kind() == NavEdgeKind::Walk
                && nav_map.has_line_of_sight(
                &TileCoords::from(grandparent),
                edge.0.end(),
                clearance_half_width,
                clearance_height,
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

            let next_heuristic_cost = next_pos.distance(*target) as u32;

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
