use crate::characters::npc::ai::AiState;
use crate::characters::npc::ai::pathfinding::pathfinder::{PathfindPending, Pathfinder, Waypoints};
use crate::characters::npc::ai::pathfinding::wander::RandomWander;
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
}

pub(super) fn pathfinder_scene() -> impl Scene {
    bsn! [
        Pathfinder
        RandomWander
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