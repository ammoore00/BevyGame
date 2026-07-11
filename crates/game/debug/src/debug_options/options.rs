use bevy::prelude::*;
use common::dev_tools::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<NavMapNodes>();
    app.init_resource::<NavMapEdges>();
}

//------ Global Debug ------//

// Navigation

#[derive(Resource, Default, DebugOption, Reflect)]
#[reflect(Resource)]
pub struct NavMapNodes(bool);

#[derive(Resource, Default, DebugOption, Reflect)]
#[reflect(Resource)]
pub struct NavMapEdges(bool);

// Physics

// User Interface

//------ Per-Entity Debug ------//

// Navigation

#[derive(Component, Default, DebugOption, Reflect)]
#[reflect(Component)]
pub struct NpcPath(bool);