use std::time::Duration;
use bevy::prelude::*;
use rand::{Rng, RngExt};
use crate::{AppSystems, PausableSystems};
use crate::game::character::state::{get_state, try_set_state, ActionState, ActionStateTracker, ActionStateEvent};
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
        DefaultWander::default(),
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

const DEFAULT_WANDER_RANGE: u32 = 5;
const DEFAULT_MAX_IDLE_TIME: u64 = 1;
const DEFAULT_MAX_MOVEMENT_TIME: u64 = 10;

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct DefaultWander {
    wander_range: u32,
    max_idle_time: Duration,
    max_movement_time: Duration,
    current_time_in_state: Duration,
}
impl Default for DefaultWander {
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
    pathfinder_query: Query<(&mut Pathfinder, &mut DefaultWander, &WorldPosition)>,
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

                if let Some(target) = select_random_wander_target(
                    nav_map,
                    &tile_coords.into(),
                    wander.wander_range,
                    rand::rng()
                ) {
                    wander.current_time_in_state = Duration::ZERO;
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
fn select_random_wander_target(
    nav_map: &TileNavMap,
    start: &TileCoords,
    _range: u32,
    mut rand: impl Rng,
) -> Option<TileCoords> {
    // TODO: Implement actual ranged pathfinding instead of only simple adjacency
    if !nav_map.has_node(start.clone()) {
        error!("Invalid start position for wander target selection: {:?}", start);
        return None;
    }

    let edges = nav_map.get_edges_from_tile(start.clone())?;
    let idx = rand.random_range(..edges.len());

    let edge = edges[idx].0;
    Some(edge.end().clone())
}