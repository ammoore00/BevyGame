use crate::characters::npc::ai::pathfinding::wander::RandomWander;
use crate::LevelLoadedSystems;
use bevy::prelude::*;
use common::{
    AppSystems, GameplaySystems, PausableSystems,
};
use crate::characters::npc::ai::pathfinding::path::Pathfinder;

mod wander;
mod path;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((path::plugin, wander::plugin,));

    app.configure_sets(
        Update,
        (PathfindingSystems::Find, PathfindingSystems::Execute)
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
}

pub(super) fn pathfinder_scene() -> impl Scene {
    bsn! [
        Pathfinder
        RandomWander
    ]
}