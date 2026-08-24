use crate::characters::npc::ai::pathfinding::pathfinder::{
    PathfindPending, Pathfinder, PathfinderClearance, Waypoints,
};
use crate::characters::npc::ai::{AiState, AiSystems};
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
#[cfg(test)]
use common::WorldCoords;
use common::WorldPosition;
use physics::Collider;
use strategy::wander::{WanderData, Wandering};

mod movement;
mod pathfinder;
mod strategy;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, pathfinder::plugin, strategy::plugin));

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
    /// Collect any generated requests
    Collect,
}

pub(super) fn pathfinder_scene() -> impl Scene {
    bsn! [
        Pathfinder
        WanderData
        Wandering
    ]
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct PathfinderData {
    pub entity: Entity,

    pub pathfinder: &'static mut Pathfinder,
    pub pending_task: Option<&'static mut PathfindPending>,
    pub waypoints: Option<&'static mut Waypoints>,

    pub pos: &'static WorldPosition,
    pub collider: &'static Collider,
    pub ai_state: &'static AiState,
}
impl PathfinderDataItem<'_, '_> {
    pub fn clearance(&self) -> PathfinderClearance {
        let collider_size = self.collider.size();
        let half_width = collider_size.x.max(collider_size.z) / 2.0;

        let height = collider_size.y;

        PathfinderClearance { half_width, height }
    }
}

/// Instantiate components required for a valid PathfinderData query for use in tests
#[cfg(test)]
pub fn pathfinder_test_components() -> impl Bundle {
    let pos = WorldCoords(Vec3::ZERO);

    (
        Pathfinder::default(),
        AiState::default(),
        Collider::cuboid(Vec3::ONE, pos),
        WorldPosition(pos),
    )
}
