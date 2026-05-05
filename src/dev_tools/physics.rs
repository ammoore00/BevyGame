use bevy::prelude::*;
use crate::datagen_api::components::{Collider, ColliderType, PhysicsData};
use crate::dev_tools::debug_options::RenderPhysicsState;
use crate::game::level::grid::coords::{ScreenCoords, TilePosition, WorldCoords, WorldPosition};
use crate::Scale;

const STATIC_COLLIDER_COLOR: Color = Color::srgb(0.2, 0.8, 1.0);
const KINEMATIC_COLLIDER_COLOR: Color = Color::srgb(0.2, 1.0, 0.3);
const CONVEX_HULL_COLOR: Color = Color::srgb(1.0, 0.8, 0.2);

const COLLIDER_Z_OFFSET: f32 = 0.25;
const COLLIDER_LINE_THICKNESS: f32 = 2.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        render_physics_colliders
    );

    app.init_gizmo_group::<PhysicsRendererGizmoGroup>();
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct PhysicsRendererGizmoGroup {}

#[derive(Component)]
struct PhysicsColliderDebugVisual;

fn render_physics_colliders(
    mut commands: Commands,
    scale: Res<Scale>,
    state: Res<State<RenderPhysicsState>>,
    debug_visual_query: Query<Entity, With<PhysicsColliderDebugVisual>>,
    tile_query: Query<(&Collider, &TilePosition, Option<&PhysicsData>)>,
    entity_query: Query<(&Collider, &WorldPosition, Option<&PhysicsData>)>,
) {
    for entity in &debug_visual_query {
        commands.entity(entity).despawn();
    }

    if state.tiles() {
        for (collider, tile_position, physics_data) in &tile_query {
            let position = tile_position.0.as_vec3();

            draw_collider(
                &mut commands,
                collider,
                position,
                physics_data,
                scale.0,
            );
        }
    }

    if state.entities() {
        for (collider, world_position, physics_data) in &entity_query {
            draw_collider(
                &mut commands,
                collider,
                world_position.as_vec3(),
                physics_data,
                scale.0,
            );
        }
    }
}

fn draw_collider(
    commands: &mut Commands,
    collider: &Collider,
    position: Vec3,
    physics_data: Option<&PhysicsData>,
    scale: f32,
) {
    let color = physics_color(physics_data);

    match collider.collider_type() {
        ColliderType::Cuboid(cuboid) => {
            draw_cuboid(commands, position, cuboid.half_extents, color, scale);
        }
        ColliderType::Capsule(capsule) => {
            draw_capsule(commands, position, capsule, color, scale);
        }
        ColliderType::ConvexHull {
            vertices,
            indices,
            ..
        } => {
            draw_convex_hull(commands, position, vertices, indices, CONVEX_HULL_COLOR, scale);
        }
    }
}

fn physics_color(physics_data: Option<&PhysicsData>) -> Color {
    match physics_data {
        Some(PhysicsData::Static) => STATIC_COLLIDER_COLOR,
        Some(PhysicsData::Kinematic { grounded: true, .. }) => KINEMATIC_COLLIDER_COLOR,
        Some(PhysicsData::Kinematic { grounded: false, .. }) => KINEMATIC_COLLIDER_COLOR,
        None => Color::WHITE,
    }
}

fn project_physics_point(point: Vec3, scale: f32) -> Vec3 {
    let screen_coords = ScreenCoords::from(WorldCoords::from(point));

    Vec3::new(
        screen_coords.x * scale,
        screen_coords.y * scale,
        screen_coords.z + COLLIDER_Z_OFFSET,
    )
}

fn draw_screen_line(
    commands: &mut Commands,
    a: Vec3,
    b: Vec3,
    color: Color,
) {
    let delta = b - a;
    let length = delta.xy().length();

    if length <= f32::EPSILON {
        return;
    }

    let midpoint = (a + b) / 2.0;
    let angle = delta.y.atan2(delta.x);

    commands.spawn((
        PhysicsColliderDebugVisual,
        Sprite {
            color,
            custom_size: Some(Vec2::new(length, COLLIDER_LINE_THICKNESS)),
            ..default()
        },
        Transform {
            translation: midpoint,
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
    ));
}

fn draw_projected_line(
    commands: &mut Commands,
    a: Vec3,
    b: Vec3,
    color: Color,
    scale: f32,
) {
    let a = project_physics_point(a, scale);
    let b = project_physics_point(b, scale);

    draw_screen_line(commands, a, b, color);
}

fn draw_projected_camera_facing_circle(
    commands: &mut Commands,
    center: Vec3,
    radius: f32,
    color: Color,
    scale: f32,
) {
    let horizontal = Vec3::new(1.0, 0.0, -1.0).normalize();
    let vertical = Vec3::Y;
    let segments = 24;

    for index in 0..segments {
        let start_angle = index as f32 / segments as f32 * std::f32::consts::TAU;
        let end_angle = (index + 1) as f32 / segments as f32 * std::f32::consts::TAU;

        let start = center
            + horizontal * start_angle.cos() * radius
            + vertical * start_angle.sin() * radius;

        let end = center
            + horizontal * end_angle.cos() * radius
            + vertical * end_angle.sin() * radius;

        draw_projected_line(commands, start, end, color, scale);
    }
}

fn draw_cuboid(
    commands: &mut Commands,
    center: Vec3,
    half_extents: Vec3,
    color: Color,
    scale: f32,
) {
    let min = center - half_extents;
    let max = center + half_extents;

    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(max.x, max.y, max.z),
        Vec3::new(min.x, max.y, max.z),
    ];

    let edges = [
        // +x face (right)
        (1, 2), (2, 6), (6, 5), (5, 1),
        // +y face (top)
        (4, 5), (5, 6), (6, 7), (7, 4),
        // +z face (front)
        (2, 3), (3, 7), (7, 6), (6, 2),
    ];

    for (a, b) in edges {
        draw_projected_line(commands, corners[a], corners[b], color, scale);
    }
}

fn draw_capsule(
    commands: &mut Commands,
    position: Vec3,
    capsule: &parry3d::shape::Capsule,
    color: Color,
    scale: f32,
) {
    let a = position + Vec3::new(
        capsule.segment.a.x,
        capsule.segment.a.y,
        capsule.segment.a.z,
    );

    let b = position + Vec3::new(
        capsule.segment.b.x,
        capsule.segment.b.y,
        capsule.segment.b.z,
    );

    let radius = capsule.radius;

    // Main capsule axis.
    draw_projected_line(commands, a, b, color, scale);

    // Cross-sections perpendicular to the vertical capsule axis.
    draw_projected_camera_facing_circle(commands, a, radius, color, scale);
    draw_projected_camera_facing_circle(commands, b, radius, color, scale);

    let horizontal = Vec3::new(1.0, 0.0, -1.0).normalize();
    let vertical = Vec3::Y;

    let offsets = [
        horizontal * radius,
        -horizontal * radius,
        vertical * radius,
        -vertical * radius,
    ];

    // Additional side lines to fill out capsule
    for offset in offsets {
        draw_projected_line(commands, a + offset, b + offset, color, scale);
    }
}

fn draw_convex_hull(
    commands: &mut Commands,
    position: Vec3,
    vertices: &[Vec3],
    indices: &[[u32; 3]],
    color: Color,
    scale: f32,
) {
    let world_vertices = vertices
        .iter()
        .map(|&vertex| position + vertex)
        .collect::<Vec<_>>();

    let view_direction = Vec3::new(1.0, 1.0, 1.0).normalize();
    let mut drawn_edges = Vec::<(usize, usize)>::new();

    for &[a, b, c] in indices {
        let a_index = a as usize;
        let b_index = b as usize;
        let c_index = c as usize;

        let Some(&a) = world_vertices.get(a_index) else {
            continue;
        };
        let Some(&b) = world_vertices.get(b_index) else {
            continue;
        };
        let Some(&c) = world_vertices.get(c_index) else {
            continue;
        };

        let normal = (b - a).cross(c - a).normalize_or_zero();

        if normal == Vec3::ZERO {
            continue;
        }

        if normal.dot(view_direction) >= 0.0 {
            continue;
        }

        draw_convex_hull_edge(
            commands,
            &world_vertices,
            &mut drawn_edges,
            a_index,
            b_index,
            color,
            scale,
        );

        draw_convex_hull_edge(
            commands,
            &world_vertices,
            &mut drawn_edges,
            b_index,
            c_index,
            color,
            scale,
        );

        draw_convex_hull_edge(
            commands,
            &world_vertices,
            &mut drawn_edges,
            c_index,
            a_index,
            color,
            scale,
        );
    }
}

fn draw_convex_hull_edge(
    commands: &mut Commands,
    vertices: &[Vec3],
    drawn_edges: &mut Vec<(usize, usize)>,
    a_index: usize,
    b_index: usize,
    color: Color,
    scale: f32,
) {
    let edge = normalized_edge(a_index, b_index);

    if drawn_edges.contains(&edge) {
        return;
    }

    let Some(&a) = vertices.get(a_index) else {
        return;
    };
    let Some(&b) = vertices.get(b_index) else {
        return;
    };

    drawn_edges.push(edge);
    draw_projected_line(commands, a, b, color, scale);
}

fn normalized_edge(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}