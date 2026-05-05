use bevy::prelude::*;
use crate::datagen_api::components::{Collider, ColliderType, PhysicsData};
use crate::dev_tools::debug_options::RenderPhysicsState;
use crate::game::level::grid::coords::{ScreenCoords, TilePosition, WorldCoords, WorldPosition};
use crate::screens::Screen;
use crate::Scale;

const STATIC_COLLIDER_COLOR: Color = Color::srgb(0.2, 0.8, 1.0);
const KINEMATIC_COLLIDER_COLOR: Color = Color::srgb(0.2, 1.0, 0.3);
const CONVEX_HULL_COLOR: Color = Color::srgb(1.0, 0.8, 0.2);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        draw_physics_colliders
            .run_if(
                in_state(RenderPhysicsState(true))
                    .and(in_state(Screen::Gameplay))
            )
    );

    app.init_gizmo_group::<PhysicsRendererGizmoGroup>();
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct PhysicsRendererGizmoGroup {}

fn project_physics_point(point: Vec3, scale: f32) -> Vec2 {
    let screen_coords = ScreenCoords::from(WorldCoords::from(point));

    Vec2::new(
        screen_coords.x * scale,
        screen_coords.y * scale,
    )
}

fn draw_projected_line(
    gizmos: &mut Gizmos<PhysicsRendererGizmoGroup>,
    a: Vec3,
    b: Vec3,
    color: Color,
    scale: f32,
) {
    gizmos.line_2d(
        project_physics_point(a, scale),
        project_physics_point(b, scale),
        color,
    );
}

fn draw_physics_colliders(
    mut gizmos: Gizmos<PhysicsRendererGizmoGroup>,
    scale: Res<Scale>,
    tile_query: Query<(&Collider, &TilePosition, Option<&PhysicsData>)>,
    entity_query: Query<(&Collider, &WorldPosition, Option<&PhysicsData>)>,
) {
    let tiles = tile_query.iter().map(|(collider, tile_pos, physics_data)| {
        (collider, WorldPosition(WorldCoords::from(&tile_pos.0)), physics_data)
    }).collect::<Vec<_>>();

    let colliders = entity_query.iter().chain(
        tiles.iter()
            .map(|(collider, world_position, physics_data)| {
                (*collider, world_position, *physics_data)
            })
    );

    for (collider, world_position, physics_data) in colliders {
        let position = world_position.as_vec3();
        let projected_position = project_physics_point(position, scale.0);

        let color = match physics_data {
            Some(PhysicsData::Static) => STATIC_COLLIDER_COLOR,
            Some(PhysicsData::Kinematic { grounded: true, .. }) => KINEMATIC_COLLIDER_COLOR,
            Some(PhysicsData::Kinematic { grounded: false, .. }) => KINEMATIC_COLLIDER_COLOR,
            None => Color::WHITE,
        };

        match collider.collider_type() {
            ColliderType::Cuboid(cuboid) => {
                draw_cuboid(&mut gizmos, position, cuboid.half_extents, color, scale.0);
            }
            ColliderType::Capsule(capsule) => {
                draw_capsule(&mut gizmos, position, capsule, color, scale.0);
            }
            ColliderType::ConvexHull(_) => {
                gizmos.circle_2d(
                    projected_position,
                    4.0 * scale.0,
                    CONVEX_HULL_COLOR,
                );
            }
        }
    }
}

fn draw_cuboid(
    gizmos: &mut Gizmos<PhysicsRendererGizmoGroup>,
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
        // Bottom face
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        // Top face
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        // Vertical edges
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    for (a, b) in edges {
        draw_projected_line(gizmos, corners[a], corners[b], color, scale);
    }
}

fn draw_capsule(
    gizmos: &mut Gizmos<PhysicsRendererGizmoGroup>,
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
    draw_projected_line(gizmos, a, b, color, scale);

    // Approximate endpoint spheres as projected 2D circles.
    gizmos.circle_2d(project_physics_point(a, scale), radius * 16.0 * scale, color);
    gizmos.circle_2d(project_physics_point(b, scale), radius * 16.0 * scale, color);

    // A few side lines make the capsule silhouette clearer.
    let offsets = [
        Vec3::X * radius,
        -Vec3::X * radius,
        Vec3::Y * radius,
        -Vec3::Y * radius,
        Vec3::Z * radius,
        -Vec3::Z * radius,
    ];

    for offset in offsets {
        draw_projected_line(gizmos, a + offset, b + offset, color, scale);
    }
}