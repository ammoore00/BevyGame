use crate::LevelLoadedSystems;
use crate::characters::npc::ai::pathfinding::path::Pathfinder;
use crate::characters::npc::ai::pathfinding::wander::RandomWander;
use bevy::prelude::*;
use common::{AppSystems, GameplaySystems, PausableSystems};

mod path;
mod wander;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((path::plugin, wander::plugin));

    app.configure_sets(
        Update,
        (
            PathfindingSystems::Find,
            PathfindingSystems::Execute,
            PathfindingSystems::Cleanup,
        )
            .chain()
            .in_set(GameplaySystems)
            .in_set(PausableSystems)
            .in_set(LevelLoadedSystems)
            .in_set(AppSystems::Update),
    );
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PathfindingSystems {
    Find,
    Execute,
    Cleanup,
}

pub(super) fn pathfinder_scene() -> impl Scene {
    bsn! [
        Pathfinder
        RandomWander
    ]
}
