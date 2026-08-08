use crate::characters::npc::ai::AiState;
use crate::characters::npc::ai::pathfinding::path::Pathfinder;
use crate::characters::npc::ai::pathfinding::wander::RandomWander;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use common::WorldPosition;
use physics::Collider;

mod follow;
mod path;
mod wander;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((follow::plugin, path::plugin, wander::plugin));
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
    pub pathfinder: &'static mut Pathfinder,
    pub pos: &'static WorldPosition,
    pub collider: &'static Collider,
    pub ai_state: &'static AiState,
}
