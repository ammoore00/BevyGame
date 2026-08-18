use crate::characters::npc::ai::pathfinding::pathfinder::{PathfindPending, Pathfinder, Waypoints};
use crate::characters::npc::ai::pathfinding::wander::{WanderData, Wandering};
use crate::characters::npc::ai::{AiState, AiSystems};
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use common::WorldPosition;
use physics::Collider;

mod follow;
mod movement;
mod pathfinder;
mod wander;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        follow::plugin,
        movement::plugin,
        pathfinder::plugin,
        wander::plugin,
    ));

    app.configure_sets(
        Update,
        (
            PathfinderSystems::Update,
            PathfinderSystems::Dispatch,
            PathfinderSystems::Collect,
        )
            .chain()
            .in_set(AiSystems::Calculate),
    );
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PathfinderSystems {
    /// Update pathfinding state
    Update,
    /// Execute dispatched pathfinding
    Dispatch,
    /// Collect any generate requests
    Collect,
}

pub(super) fn pathfinder_scene() -> impl Scene {
    bsn! [
        Pathfinder
        WanderData
        Wandering
    ]
}

pub const TARGET_REACHED_THRESHOLD: f32 = 0.2;

#[derive(QueryData)]
#[query_data(mutable)]
pub struct PathfinderQuery {
    pub entity: Entity,

    pub pathfinder: &'static mut Pathfinder,

    pub pending_task: Option<&'static PathfindPending>,
    pub waypoints: Option<&'static mut Waypoints>,

    pub pos: &'static WorldPosition,
    pub collider: &'static Collider,
    pub ai_state: &'static AiState,
}
