pub mod pathfinding;

use crate::characters::npc::ai::pathfinding::pathfinder_scene;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(pathfinding::plugin);
}

pub(super) fn ai_scene() -> impl Scene {
    bsn! [
        pathfinder_scene()
        AiState
    ]
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AiState {
    #[default]
    Wander,
    _Attack,
}