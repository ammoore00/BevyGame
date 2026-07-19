use crate::debug_options::options::{NavMapEdgesRes, NavMapNodesRes};
use crate::debug_options::render::helpers::{LineSettings, draw_world_space_circle_projected, draw_world_line};
use crate::debug_options::render::palette::{NAV_EDGE_ARROW_LENGTH, NAV_EDGE_ARROW_WIDTH, NAV_EDGE_END_PADDING, NAV_EDGE_FORWARD_COLOR, NAV_EDGE_LINE_THICKNESS, NAV_EDGE_REVERSE_COLOR, NAV_EDGE_DIRECTIONAL_OFFSET, NAV_NODE_COLOR, NAV_NODE_LINE_THICKNESS, NAV_NODE_RADIUS};
use bevy::prelude::*;
use common::dev_tools::DebugState;
use common::{GameState, Scale, marker, WorldCoords};
use runtime::debug::TileNavMap;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_nav_node_render.run_if(resource_changed::<NavMapNodesRes>),
    );
    app.add_observer(spawn_nav_node_render);
    app.add_observer(cleanup_nav_node_render);

    app.add_systems(
        Update,
        update_nav_edge_render.run_if(resource_changed::<NavMapEdgesRes>),
    );
    app.add_observer(spawn_nav_edge_render);
    app.add_observer(cleanup_nav_edge_render);
}

//------ Nodes ------//

marker!(NavNodeRender);

#[derive(Event)]
struct SpawnNavNodes;
#[derive(Event)]
struct CleanupNavNodes;

fn update_nav_node_render(render_nav_nodes: Res<NavMapNodesRes>, mut commands: Commands) {
    if render_nav_nodes.get() {
        commands.trigger(SpawnNavNodes);
    } else {
        commands.trigger(CleanupNavNodes);
    }
}

fn spawn_nav_node_render(
    _: On<SpawnNavNodes>,
    nav_query: Query<&TileNavMap>,
    scale: Res<Scale>,
    mut commands: Commands,
) {
    let Ok(nav) = nav_query.single() else {
        return error!("Failed to obtain nav map component");
    };

    for node in nav.debug_node_positions() {
        draw_world_space_circle_projected(
            node.into(),
            NAV_NODE_RADIUS,
            LineSettings {
                color: NAV_NODE_COLOR,
                thickness: NAV_NODE_LINE_THICKNESS,
            },
            scale.0,
        )
        .into_iter()
        .for_each(|line| {
            commands.spawn((nav_node_bundle(), line));
        });
    }
}

fn nav_node_bundle() -> impl Bundle {
    (NavNodeRender, DespawnOnExit(GameState::Gameplay))
}

fn cleanup_nav_node_render(
    _: On<CleanupNavNodes>,
    render_query: Query<Entity, With<NavNodeRender>>,
    mut commands: Commands,
) {
    for entity in render_query.iter() {
        commands.entity(entity).despawn();
    }
}

//------ Edges ------//

marker!(NavEdgeRender);

#[derive(Event)]
struct SpawnNavEdges;
#[derive(Event)]
struct CleanupNavEdges;

fn update_nav_edge_render(render_nav_nodes: Res<NavMapEdgesRes>, mut commands: Commands) {
    if render_nav_nodes.get() {
        commands.trigger(SpawnNavEdges);
    } else {
        commands.trigger(CleanupNavEdges);
    }
}

fn spawn_nav_edge_render(
    _: On<SpawnNavEdges>,
    nav_query: Query<&TileNavMap>,
    scale: Res<Scale>,
    mut commands: Commands,
) {
    let Ok(nav) = nav_query.single() else {
        return error!("Failed to obtain nav map component");
    };

    let line_scale = scale.0;

    for (start, end) in nav.debug_edge_segments() {
        let delta = end - start;
        let original_length = delta.length();
        let direction = delta.normalize_or_zero();

        if direction == Vec3::ZERO {
            continue;
        }

        // FIX: Prevent line inversion.
        // If the line is shorter than our paddings combined with the arrow head,
        // the math will cause the start and end points to cross over and shoot backwards.
        let required_length = (NAV_EDGE_END_PADDING * 2.0) + NAV_EDGE_ARROW_LENGTH;
        if original_length <= required_length {
            continue;
        }

        let color = if delta.x > 0.0 || (delta.x == 0.0 && delta.z >= 0.0) {
            NAV_EDGE_FORWARD_COLOR
        } else {
            NAV_EDGE_REVERSE_COLOR
        };

        let settings = LineSettings {
            color,
            thickness: NAV_EDGE_LINE_THICKNESS,
        };

        let perpendicular = Vec3::Y.cross(direction).normalize_or_zero();
        let offset = perpendicular * NAV_EDGE_DIRECTIONAL_OFFSET;

        let start_adj = start + offset + (direction * NAV_EDGE_END_PADDING);
        let end_adj = end + offset - (direction * NAV_EDGE_END_PADDING);

        // 1. Draw the main segment
        commands.spawn((
            nav_edge_bundle(),
            draw_world_line(WorldCoords(start_adj), WorldCoords(end_adj), settings, line_scale),
        ));

        let arrow_tip = end_adj;
        let arrow_base = end_adj - (direction * NAV_EDGE_ARROW_LENGTH);
        let arrow_left = arrow_base + (perpendicular * NAV_EDGE_ARROW_WIDTH);
        let arrow_right = arrow_base - (perpendicular * NAV_EDGE_ARROW_WIDTH);

        // 2. Draw the left arrowhead line
        commands.spawn((
            nav_edge_bundle(),
            draw_world_line(WorldCoords(arrow_tip), WorldCoords(arrow_left), settings, line_scale),
        ));

        // 3. Draw the right arrowhead line
        commands.spawn((
            nav_edge_bundle(),
            draw_world_line(WorldCoords(arrow_tip), WorldCoords(arrow_right), settings, line_scale),
        ));
    }
}

fn nav_edge_bundle() -> impl Bundle {
    (NavEdgeRender, DespawnOnExit(GameState::Gameplay))
}

fn cleanup_nav_edge_render(
    _: On<CleanupNavEdges>,
    render_query: Query<Entity, With<NavEdgeRender>>,
    mut commands: Commands,
) {
    for entity in render_query.iter() {
        commands.entity(entity).despawn();
    }
}

//------ Pathing ------//
