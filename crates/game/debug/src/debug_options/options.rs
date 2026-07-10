use bevy::prelude::*;
use common::dev_tools::*;

#[derive(Resource, DebugOption)]
pub struct NavMapNodes(bool);
#[derive(Resource, DebugOption)]
pub struct NavMapEdges(bool);

#[derive(Component, DebugOption)]
pub struct NpcPath(bool);