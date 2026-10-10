use crate::LevelLoadedSystems;
use crate::characters::npc::ai::pathfinding::pathfinder_scene;
use crate::characters::npc::ai::state::state_scene;
use bevy::prelude::*;
use common::{AppSystems, Facing, GameplaySystems, PausableSystems, WorldCoords};

mod movement;
// TODO: Remove this pub
pub mod pathfinding;
pub mod state;

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

pub(super) fn ai_scene(pos: Vec3) -> impl Scene {
    bsn! {
        @pathfinder_scene()
        @state_scene()
        NpcHome { pos }
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AiSystems {
    Calculate,
    Execute,
    Cleanup,
}

#[derive(Component, Debug, Clone, Copy, Default)]
struct NpcHome {
    pos: WorldCoords,
    facing: Facing,
}
