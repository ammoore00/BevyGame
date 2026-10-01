use crate::LevelLoadedSystems;
use crate::characters::npc::ai::pathfinding::pathfinder_scene;
use crate::characters::npc::ai::state::state_scene;
use bevy::prelude::*;
use common::{AppSystems, GameplaySystems, PausableSystems};

mod movement;
// TODO: Remove this pub
pub mod pathfinding;
mod state;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, pathfinding::plugin, state::plugin));

    app.configure_sets(
        Update,
        (AiSystems::Calculate, AiSystems::Execute, AiSystems::Cleanup)
            .chain()
            .in_set(GameplaySystems)
            .in_set(PausableSystems)
            .in_set(LevelLoadedSystems)
            .in_set(AppSystems::Update),
    );
}

pub(super) fn ai_scene() -> impl Scene {
    bsn! [
        pathfinder_scene()
        state_scene()
    ]
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AiSystems {
    Calculate,
    Execute,
    Cleanup,
}
