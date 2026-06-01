use crate::dev_tools::debug_menu::debug_options::{RenderNPCPathsState, RenderNavMapEdgesState, RenderNavMapNodesState};
use bevy::prelude::*;
use crate::datagen_api::ai::pathfinding::{Pathfinder, PathfinderState};
use crate::dev_tools::debug_menu::level_render::{draw_projected_camera_facing_circle, draw_screen_line, project_point};
use crate::game::level::grid::coords::WorldPosition;
use crate::game::level::grid::nav::TileNavMap;
use crate::Scale;

const NAV_NODE_COLOR: Color = Color::srgb(
    0.55,
    0.95,
    0.80,
);
const NAV_EDGE_FORWARD_COLOR: Color = Color::srgb(
    0.30,
    0.60,
    0.95,
);
const NAV_EDGE_REVERSE_COLOR: Color = Color::srgb(
    0.20,
    0.75,
    0.80,
);

const PATH_COLOR: Color = Color::srgb(
    0.95,
    0.85,
    0.30,
);

const NAV_NODE_RADIUS: f32 = 0.5;
const NAV_NODE_LINE_THICKNESS: f32 = 3.0;

const NAV_EDGE_LINE_THICKNESS: f32 = 4.0;
const NAV_EDGE_SCREEN_OFFSET: f32 = 8.0;
const NAV_EDGE_END_PADDING: f32 = 12.0;
const NAV_EDGE_ARROW_LENGTH: f32 = 10.0;
const NAV_EDGE_ARROW_WIDTH: f32 = 6.0;

const NAV_DEBUG_Y_OFFSET: f32 = -0.45;

pub(in crate::dev_tools) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            render_nav_nodes,
            render_nav_edges,
            render_npc_paths,
        ),
    );
}

#[derive(Component, Clone, Copy)]
struct NPCPathDebugVisual;

fn render_npc_paths(
    mut commands: Commands,
    scale: Res<Scale>,
    state: Res<State<RenderNPCPathsState>>,
    debug_visual_query: Query<Entity, With<NPCPathDebugVisual>>,
    path_query: Query<(&Pathfinder, &WorldPosition)>,
) {
    for entity in &debug_visual_query {
        commands.entity(entity).despawn();
    }

    if !state.0 {
        return;
    }

    for (pathfinder, pos) in path_query {
        if let PathfinderState::Moving(target) = pathfinder.state() {
            let tile_path = target.get();
            let remaining_path = tile_path.get_remaining_path();

            for i in 0..remaining_path.len() {
                let current = &remaining_path[i];

                draw_projected_camera_facing_circle(
                    &mut commands,
                    **current + Vec3::Y * (1. + NAV_DEBUG_Y_OFFSET),
                    NAV_NODE_RADIUS / 3.,
                    PATH_COLOR,
                    scale.0,
                    NAV_NODE_LINE_THICKNESS,
                    NavNodeDebugVisual,
                );

                if i == 0 {
                    draw_directed_nav_edge(
                        &mut commands,
                        *pos.0 + Vec3::Y * (NAV_DEBUG_Y_OFFSET),
                        **current + Vec3::Y * (1. + NAV_DEBUG_Y_OFFSET),
                        scale.0,
                        NavEdgeColor::Color(PATH_COLOR),
                    );
                }

                if i < remaining_path.len() - 1 {
                    let next = &remaining_path[i + 1];
                    draw_directed_nav_edge(
                        &mut commands,
                        **current + Vec3::Y * (1. + NAV_DEBUG_Y_OFFSET),
                        **next + Vec3::Y * (1. + NAV_DEBUG_Y_OFFSET),
                        scale.0,
                        NavEdgeColor::Color(PATH_COLOR),
                    );
                }
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
struct NavNodeDebugVisual;
#[derive(Component, Clone, Copy)]
struct NavEdgeDebugVisual;

fn render_nav_nodes(
    mut commands: Commands,
    scale: Res<Scale>,
    state: Res<State<RenderNavMapNodesState>>,
    debug_visual_query: Query<Entity, With<NavNodeDebugVisual>>,
    nav_query: Query<&TileNavMap>,
) {
    for entity in &debug_visual_query {
        commands.entity(entity).despawn();
    }

    if !state.0 {
        return;
    }

    for nav_map in nav_query {
        for position in nav_map.debug_node_positions() {
            draw_projected_camera_facing_circle(
                &mut commands,
                position + Vec3::Y * NAV_DEBUG_Y_OFFSET,
                NAV_NODE_RADIUS,
                NAV_NODE_COLOR,
                scale.0 / 2.0,
                NAV_NODE_LINE_THICKNESS,
                NavNodeDebugVisual,
            );
        }
    }
}

fn render_nav_edges(
    mut commands: Commands,
    scale: Res<Scale>,
    state: Res<State<RenderNavMapEdgesState>>,
    debug_visual_query: Query<Entity, With<NavEdgeDebugVisual>>,
    nav_query: Query<&TileNavMap>,
) {
    for entity in &debug_visual_query {
        commands.entity(entity).despawn();
    }

    if !state.0 {
        return;
    }

    for nav_map in nav_query {
        for (start, end) in nav_map.debug_edge_segments() {
            draw_directed_nav_edge(
                &mut commands,
                start + Vec3::Y * NAV_DEBUG_Y_OFFSET,
                end + Vec3::Y * NAV_DEBUG_Y_OFFSET,
                scale.0 / 2.0,
                NavEdgeColor::Auto,
            );
        }
    }
}

fn draw_directed_nav_edge(
    commands: &mut Commands,
    start: Vec3,
    end: Vec3,
    scale: f32,
    color: NavEdgeColor,
) {
    let start = project_point(start, scale);
    let end = project_point(end, scale);

    let delta = end - start;
    let direction = delta.xy().normalize_or_zero();

    if direction == Vec2::ZERO {
        return;
    }

    let color = match color {
        NavEdgeColor::Auto => nav_edge_color(delta),
        NavEdgeColor::Color(color) => color,
    };

    let perpendicular = Vec2::new(-direction.y, direction.x);
    let offset = perpendicular * NAV_EDGE_SCREEN_OFFSET;

    let start = start + offset.extend(0.0) + (direction * NAV_EDGE_END_PADDING).extend(0.0);
    let end = end + offset.extend(0.0) - (direction * NAV_EDGE_END_PADDING).extend(0.0);

    if (end - start).xy().length() <= NAV_EDGE_ARROW_LENGTH {
        return;
    }

    draw_screen_line(
        commands,
        start,
        end,
        color,
        NAV_EDGE_LINE_THICKNESS,
        NavEdgeDebugVisual,
    );

    let arrow_tip = end;
    let arrow_base = end - (direction * NAV_EDGE_ARROW_LENGTH).extend(0.0);
    let arrow_left = arrow_base + (perpendicular * NAV_EDGE_ARROW_WIDTH).extend(0.0);
    let arrow_right = arrow_base - (perpendicular * NAV_EDGE_ARROW_WIDTH).extend(0.0);

    draw_screen_line(
        commands,
        arrow_tip,
        arrow_left,
        color,
        NAV_EDGE_LINE_THICKNESS,
        NavEdgeDebugVisual,
    );

    draw_screen_line(
        commands,
        arrow_tip,
        arrow_right,
        color,
        NAV_EDGE_LINE_THICKNESS,
        NavEdgeDebugVisual,
    );
}

enum NavEdgeColor {
    Auto,
    Color(Color),
}

fn nav_edge_color(delta: Vec3) -> Color {
    if delta.x > 0.0 || delta.x == 0.0 && delta.y >= 0.0 {
        NAV_EDGE_FORWARD_COLOR
    } else {
        NAV_EDGE_REVERSE_COLOR
    }
}