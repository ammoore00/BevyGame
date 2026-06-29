use crate::dev_tools::debug_menu::debug_options::{RenderPhysicsEntitiesState, RenderPhysicsTilesState};
use crate::game::level::grid::coords::{TilePosition, WorldPosition};
use crate::Scale;
use bevy::prelude::*;
use crate::dev_tools::debug_menu::level_render::{draw_capsule, draw_convex_hull, draw_cuboid};
use crate::game::physics::components::{Collider, ColliderType, PhysicsData};
use crate::game::physics::math::ToBevy;

const KINEMATIC_COLLIDER_COLOR: Color = Color::srgb(
    0.90,
    0.75,
    0.35,
);
const STATIC_COLLIDER_COLOR: Color = Color::srgb(
    0.90,
    0.35,
    0.35,
);
const CONVEX_HULL_COLOR: Color = Color::srgb(
    0.95,
    0.55,
    0.30,
);

const COLLIDER_LINE_THICKNESS: f32 = 2.0;

pub(in crate::dev_tools) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        render_physics_colliders
    );
}

#[derive(Component, Debug, Clone, Copy)]
struct PhysicsColliderDebugVisual;

fn render_physics_colliders(
    mut commands: Commands,
    scale: Res<Scale>,
    entity_state: Res<State<RenderPhysicsEntitiesState>>,
    physics_state: Res<State<RenderPhysicsTilesState>>,
    debug_visual_query: Query<Entity, With<PhysicsColliderDebugVisual>>,
    tile_query: Query<(&Collider, &TilePosition, Option<&PhysicsData>)>,
    entity_query: Query<(&Collider, &WorldPosition, Option<&PhysicsData>)>,
) {
    for entity in &debug_visual_query {
        commands.entity(entity).despawn();
    }

    if physics_state.0 {
        for (collider, tile_position, physics_data) in &tile_query {
            let position = tile_position.0.as_vec3();

            draw_collider(
                &mut commands,
                collider,
                position,
                scale.0,
                physics_color(physics_data),
            );
        }
    }

    if entity_state.0 {
        for (collider, world_position, physics_data) in &entity_query {
            draw_collider(
                &mut commands,
                collider,
                world_position.as_vec3(),
                scale.0,
                physics_color(physics_data),
            );
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

fn draw_collider(
    commands: &mut Commands,
    collider: &Collider,
    position: Vec3,
    scale: f32,
    color: Color,
) {
    match collider.collider_type() {
        ColliderType::Cuboid(cuboid) => {
            draw_cuboid(commands, position, cuboid.half_extents.to_bevy(), color, scale, COLLIDER_LINE_THICKNESS, PhysicsColliderDebugVisual);
        }
        ColliderType::Capsule(capsule) => {
            draw_capsule(commands, position, capsule, color, scale, COLLIDER_LINE_THICKNESS, PhysicsColliderDebugVisual);
        }
        ColliderType::ConvexHull {
            vertices,
            indices,
            ..
        } => {
            draw_convex_hull(commands, position, vertices, indices, CONVEX_HULL_COLOR, scale, COLLIDER_LINE_THICKNESS, PhysicsColliderDebugVisual);
        }
    }
}