//! Handle player input and translate it into movement through a characters
//! controller. A characters controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the characters controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   characters.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the characters within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use crate::collision::PhysicsCollisionsProcessedEvent;
use crate::components::{Collider, CollisionContact, KinematicData, PhysicsData};
use bevy::ecs::error::error;
use bevy::prelude::*;
use common::{AppSystems, Facing, PausableSystems, WorldCoords, WorldPosition};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            set_intended_velocity,
            apply_gravity,
            update_facing_from_movement,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

    app.add_observer(process_collisions);
}

pub const GRAVITY: f32 = 0.5;
pub const STEP_UP_HEIGHT: f32 = 0.3;

pub const MAX_STABLE_SLOPE_ANGLE: f32 = 45.0_f32.to_radians();

pub const DEFAULT_MAX_SPEED: f32 = 2.0;

/// These are the movement parameters for our character's controller.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec3,
    pub sprinting: bool,
    /// Maximum speed in meters per second.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec3::ZERO,
            sprinting: false,
            max_speed: DEFAULT_MAX_SPEED,
        }
    }
}

fn set_intended_velocity(time: Res<Time>, query: Query<(&MovementController, &mut PhysicsData)>) {
    for (controller, mut physics) in query {
        if let PhysicsData::Kinematic(KinematicData {
            ref mut next_displacement,
            ..
        }) = *physics
        {
            let mut intent = controller.intent * controller.max_speed * time.delta_secs();

            if controller.sprinting {
                intent *= Vec3::new(1.5, 1.0, 1.5);
            }

            next_displacement.x = intent.x;
            next_displacement.z = intent.z;
            next_displacement.y += intent.y;
        }
    }
}

/// Applies gravity to the characters’s current displacement.
fn apply_gravity(query: Query<&mut PhysicsData>, time: Res<Time>) {
    for mut physics in query {
        if let PhysicsData::Kinematic(KinematicData {
            ref mut next_displacement,
            ..
        }) = *physics
        {
            next_displacement.y -= GRAVITY * time.delta_secs();
        }
    }
}

/// Update the facing angle for characters based on their intended movement
fn update_facing_from_movement(query: Query<(&PhysicsData, &mut Facing)>) {
    for (physics, mut facing) in query {
        let PhysicsData::Kinematic(KinematicData {
            next_displacement, ..
        }) = *physics
        else {
            continue;
        };

        let ground_movement = next_displacement.xz();

        if ground_movement.length() > 1e-6 {
            *facing = Facing::from(ground_movement);
        }
    }
}

fn process_collisions(
    event: On<PhysicsCollisionsProcessedEvent>,
    mut query: Query<(&mut PhysicsData, &mut WorldPosition, &Collider)>,
    other_collider_query: Query<(Entity, &Collider), With<PhysicsData>>,
    time: Res<Time>,
) {
    let Ok((mut physics, mut pos, collider)) = query.get_mut(event.entity) else {
        return error!(
            "Failed to get physics data for event entity {:?}",
            event.entity
        );
    };

    let mut grounded = false;
    let mut ground_normal = Vec3::ZERO;

    let PhysicsData::Kinematic(ref mut kinematic_data) = *physics else {
        return error!("Collision event triggered for non-kinematic physics object!");
    };

    for collision in event.physics_collisions.iter() {
        let Ok((_, other_collider)) = other_collider_query.get(collision.other_entity) else {
            return error!(
                "Failed to get collider for colliding entity {:?}",
                collision.other_entity
            );
        };

        let all_other_colliders = other_collider_query
            .iter()
            .filter(|(other_entity, _)| *other_entity != event.entity)
            .map(|(_, other_collider)| other_collider)
            .collect::<Vec<_>>();

        // Get the collision response from this collision
        // This includes the displacement offset to apply
        //  and the ground normal if this is a grounded collision
        let collision_response = get_collision_response(
            &collision.contact,
            kinematic_data,
            collider,
            &all_other_colliders,
            pos.0,
        );

        kinematic_data.next_displacement += collision_response.displacement_from_collision;
        if let Some(new_ground_normal) = collision_response.grounded_normal
            && new_ground_normal.y > ground_normal.y
        {
            ground_normal = new_ground_normal;
            grounded = true;
        }
    }

    update_ground_state(kinematic_data, pos.0, grounded, ground_normal, &time);
    apply_movement(&physics, &mut pos);
}

struct CollisionResponse {
    displacement_from_collision: Vec3,
    grounded_normal: Option<Vec3>,
}

fn get_collision_response(
    collision: &CollisionContact,
    kinematic_data: &KinematicData,
    collider: &Collider,
    all_other_colliders: &[&Collider],
    pos: WorldCoords,
) -> CollisionResponse {
    let normal = collision.normal();
    let (grounded_normal, ground_collision) = if normal.y > 0.7 {
        (Some(normal), true)
    } else {
        (None, false)
    };

    let velocity_along_normal = kinematic_data.next_displacement.dot(normal);

    // If we aren't traveling into the collider at all, then we don't need to offset the displacement
    if velocity_along_normal >= 0.0 {
        return CollisionResponse {
            displacement_from_collision: Vec3::ZERO,
            grounded_normal,
        };
    }

    let collision_displacement = -normal * velocity_along_normal;
    let displacement_from_collision = if !ground_collision && kinematic_data.grounded {
        // If this is a horizontal (e.g., wall) collision, try to step up
        // If we fail to step up, treat the collision as normal
        if let Some(step_up) = try_step_up(collider, all_other_colliders, pos) {
            step_up
        } else {
            collision_displacement
        }
    } else {
        collision_displacement
    };

    CollisionResponse {
        displacement_from_collision,
        grounded_normal,
    }
}

fn try_step_up(
    collider: &Collider,
    all_other_colliders: &[&Collider],
    pos: WorldCoords,
) -> Option<Vec3> {
    for test_height in 1..=10 {
        let test_step = (test_height as f32) * (STEP_UP_HEIGHT / 10.0);
        let test_position = pos.0 + Vec3::Y * test_step;

        let test_collider =
            Collider::with_collider(collider.collider_type().clone(), test_position);

        let still_colliding = all_other_colliders
            .iter()
            .any(|other_collider| test_collider.check_collision(other_collider).is_some());

        if !still_colliding {
            return Some(Vec3::Y * test_step);
        }
    }

    None
}

fn update_ground_state(
    kinematic_data: &mut KinematicData,
    pos: WorldCoords,
    grounded: bool,
    ground_normal: Vec3,
    time: &Time,
) {
    kinematic_data.grounded = grounded;

    if kinematic_data.grounded {
        kinematic_data.time_since_grounded = 0.0;
        kinematic_data.last_grounded_height = pos.y;

        let slope_displacement = stabilize_on_slope(ground_normal, time);
        kinematic_data.next_displacement += slope_displacement;
    } else {
        kinematic_data.time_since_grounded += time.delta_secs();
    }
}

fn stabilize_on_slope(ground_normal: Vec3, time: &Time) -> Vec3 {
    if ground_normal == Vec3::ZERO {
        return Vec3::ZERO;
    }

    // Skip too-steep slopes.
    let slope_angle = ground_normal.angle_between(Vec3::Y);
    if slope_angle > MAX_STABLE_SLOPE_ANGLE {
        return Vec3::ZERO;
    }

    // 1) Compute how much gravity moved us this frame (world displacement caused by gravity).
    //    This must match how you apply gravity in apply_gravity (i.e. GRAVITY * delta_time).
    let gravity_frame = Vec3::new(0.0, -GRAVITY, 0.0) * time.delta_secs();

    // 2) Split that gravity displacement into normal and tangential parts relative to the ground.
    let gravity_normal_comp = ground_normal * gravity_frame.dot(ground_normal);
    let gravity_tangential = gravity_frame - gravity_normal_comp; // this is the downslope vector gravity would cause

    // 3) Subtract THAT tangential gravity contribution from the final displacement.
    //    This removes the passive sliding caused by gravity this frame while preserving
    //    any non-gravity movement (player input, step push).
    let mut displacement = -gravity_tangential;

    // 4) Safety: remove penetration into the surface if any remains.
    let into_surface = displacement.dot(ground_normal);
    if into_surface < 0.0 {
        displacement -= ground_normal * into_surface;
    }

    displacement
}

/// Apply the final physics displacement to the entity's position
fn apply_movement(physics: &PhysicsData, position: &mut WorldPosition) {
    let new_position = if let PhysicsData::Kinematic(KinematicData {
        next_displacement, ..
    }) = *physics
    {
        position.as_vec3() + next_displacement
    } else {
        return;
    };
    position.set(new_position);
}

fn check_collisions(
    time: Res<Time>,
    query: Query<(Entity, &mut PhysicsData, &Collider, &WorldPosition)>,
    collider_query: Query<(Entity, &Collider)>,
) {
    for (entity, mut physics, collider, position) in query {
        if let PhysicsData::Kinematic(KinematicData {
            ref mut next_displacement,
            ref mut grounded,
            ref mut time_since_grounded,
            ref mut last_grounded_height,
        }) = *physics
        {
            let current_position = position.as_vec3();
            let mut detected_ground_collision = false;

            let mut ground_normal = Vec3::ZERO;

            for (other_entity, other_collider) in collider_query {
                if entity == other_entity {
                    continue;
                }

                if let Some(collision) = Collider::check_collision(collider, other_collider) {
                    let new_ground_normal = handle_collision_response(
                        collision,
                        next_displacement,
                        &mut detected_ground_collision,
                        *grounded,
                        collider,
                        current_position,
                        entity,
                        &collider_query,
                        time.delta_secs(),
                    );

                    if let Some(new_ground_normal) = new_ground_normal
                        && new_ground_normal.y > ground_normal.y
                    {
                        ground_normal = new_ground_normal;
                    }
                }
            }

            update_ground_state_old(
                ground_normal,
                next_displacement,
                grounded,
                time_since_grounded,
                last_grounded_height,
                detected_ground_collision,
                current_position.y,
                time.delta_secs(),
            );
        }
        //apply_movement(&physics, &mut position);
    }
}

/// Handles collision resolution depending on type (ground, wall, step, etc.)
fn handle_collision_response(
    collision: CollisionContact,
    displacement: &mut Vec3,
    detected_ground_collision: &mut bool,
    grounded: bool,
    collider: &Collider,
    current_position: Vec3,
    entity: Entity,
    query2: &Query<(Entity, &Collider)>,
    delta_time: f32,
) -> Option<Vec3> {
    let normal = collision.normal();
    let velocity_along_normal = displacement.dot(normal);
    let is_horizontal = normal.y.abs() < 0.7;

    // Detect ground contact
    let grounded_normal = if normal.y > 0.7 {
        *detected_ground_collision = true;
        Some(normal)
    } else {
        None
    };

    if is_horizontal && velocity_along_normal < 0.0 && grounded {
        // Try stepping up first
        if !try_step_up_old(
            collider,
            current_position,
            entity,
            query2,
            displacement,
            delta_time,
        ) {
            *displacement -= normal * velocity_along_normal;
        }
    } else if velocity_along_normal < 0.0 {
        *displacement -= normal * velocity_along_normal;
    };

    grounded_normal
}

/// Attempts to "step up" a small ledge if possible.
/// Returns `true` if a valid step-up position was found and applied.
fn try_step_up_old(
    collider: &Collider,
    current_position: Vec3,
    entity: Entity,
    query2: &Query<(Entity, &Collider)>,
    displacement: &mut Vec3,
    delta_time: f32,
) -> bool {
    for test_height in 1..=10 {
        let test_step = (test_height as f32) * (STEP_UP_HEIGHT / 10.0);
        let test_position = current_position + Vec3::Y * test_step;

        let test_collider =
            Collider::with_collider(collider.collider_type().clone(), test_position);

        let still_colliding = query2
            .iter()
            .filter(|(other_entity, _)| *other_entity != entity)
            .any(|(_, other_collider)| test_collider.check_collision(other_collider).is_some());

        if !still_colliding {
            displacement.y = displacement.y.max(test_step * delta_time);
            return true;
        }
    }
    false
}

/// Updates grounded state and timer based on whether the entity was grounded this frame.
fn update_ground_state_old(
    ground_normal: Vec3,
    displacement: &mut Vec3,
    grounded: &mut bool,
    time_since_grounded: &mut f32,
    last_grounded_height: &mut f32,
    detected_ground_collision: bool,
    current_height: f32,
    delta_time: f32,
) {
    *grounded = detected_ground_collision;

    if *grounded {
        *time_since_grounded = 0.0;
        *last_grounded_height = current_height;
        stabilize_on_slope_old(displacement, ground_normal, delta_time);
    } else {
        *time_since_grounded += delta_time;
    }
}

/// Stops sliding by removing only the gravity-produced tangential displacement
/// along the slope for this frame. Leaves player input intact.
fn stabilize_on_slope_old(displacement: &mut Vec3, ground_normal: Vec3, delta_time: f32) {
    if ground_normal == Vec3::ZERO {
        return;
    }

    // Skip too-steep slopes.
    let slope_angle = ground_normal.angle_between(Vec3::Y);
    if slope_angle > MAX_STABLE_SLOPE_ANGLE {
        return;
    }

    // 1) Compute how much gravity moved us this frame (world displacement caused by gravity).
    //    This must match how you apply gravity in apply_gravity (i.e. GRAVITY * delta_time).
    let gravity_frame = Vec3::new(0.0, -GRAVITY, 0.0) * delta_time;

    // 2) Split that gravity displacement into normal and tangential parts relative to the ground.
    let gravity_normal_comp = ground_normal * gravity_frame.dot(ground_normal);
    let gravity_tangential = gravity_frame - gravity_normal_comp; // this is the downslope vector gravity would cause

    // 3) Subtract THAT tangential gravity contribution from the final displacement.
    //    This removes the passive sliding caused by gravity this frame while preserving
    //    any non-gravity movement (player input, step push).
    *displacement -= gravity_tangential;

    // 4) Safety: remove penetration into the surface if any remains.
    let into_surface = displacement.dot(ground_normal);
    if into_surface < 0.0 {
        *displacement -= ground_normal * into_surface;
    }
}
