use crate::dev_tools::debug_options::{RenderNavMapEdgesState, RenderNavMapNodesState};
use bevy::prelude::*;
use crate::game::level::grid::nav::TileNavMap;
use crate::Scale;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            render_nav_nodes.run_if(in_state(RenderNavMapNodesState(true))),
            render_nav_edges.run_if(in_state(RenderNavMapEdgesState(true))),
        ),
    );
}

#[derive(Component)]
struct NavNodeDebugVisual;
#[derive(Component)]
struct NavEdgeDebugVisual;

fn render_nav_nodes(
    mut commands: Commands,
    scale: Res<Scale>,
    debug_visual_query: Query<Entity, With<NavNodeDebugVisual>>,
    nav_query: Query<&TileNavMap>,
) {

}

fn render_nav_edges(
    mut commands: Commands,
    scale: Res<Scale>,
    debug_visual_query: Query<Entity, With<NavEdgeDebugVisual>>,
    nav_query: Query<&TileNavMap>,
) {

}