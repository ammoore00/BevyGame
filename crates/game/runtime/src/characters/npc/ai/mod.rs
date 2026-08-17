pub mod pathfinding;

use crate::LevelLoadedSystems;
use crate::characters::npc::ai::pathfinding::pathfinder_scene;
use bevy::prelude::*;
use common::{AppSystems, GameplaySystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(pathfinding::plugin);

    app.configure_sets(
        Update,
        (AiSystems::Calculate, AiSystems::Execute, AiSystems::Cleanup)
            .chain()
            .in_set(GameplaySystems)
            .in_set(PausableSystems)
            .in_set(LevelLoadedSystems)
            .in_set(AppSystems::Update),
    );

    app.add_systems(Update, update_prev_state.in_set(AiSystems::Cleanup));
}

pub(super) fn ai_scene() -> impl Scene {
    bsn! [
        pathfinder_scene()
        AiState
    ]
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AiSystems {
    Calculate,
    Execute,
    Cleanup,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AiState {
    current: AiStateKind,
    prev: AiStateKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AiStateKind {
    #[default]
    Wander,
    _Follow,
}

fn update_prev_state(query: Query<&mut AiState>) {
    for mut ai_state in query {
        ai_state.prev = ai_state.current;
    }
}
