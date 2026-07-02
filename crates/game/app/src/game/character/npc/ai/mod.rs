pub mod pathfinding;

use crate::game::character::npc::ai::pathfinding::pathfinder_scene;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(pathfinding::plugin);
}

pub(super) fn ai_scene() -> impl Scene {
    bsn! [
        pathfinder_scene()
    ]
}