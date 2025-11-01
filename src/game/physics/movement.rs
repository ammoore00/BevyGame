//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::prelude::*;

use crate::game::grid::coords::WorldPosition;
use crate::game::physics::components::{Collider, ColliderType, CollisionEvent, PhysicsData};
use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (set_intended_velocity, check_collisions, apply_movement)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec3,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec3::ZERO,
            max_speed: 3.5,
        }
    }
}

fn set_intended_velocity(time: Res<Time>, query: Query<(&MovementController, &mut PhysicsData)>) {
    for (controller, mut physics) in query {
        if let PhysicsData::Kinematic {
            displacement: ref mut velocity,
            ..
        } = *physics
        {
            let intent = controller.intent * controller.max_speed * time.delta_secs();
            velocity.x = intent.x;
            velocity.z = intent.z;
            velocity.y += intent.y;
        }
    }
}

pub const GRAVITY: f32 = 1.0;
pub const STEP_UP_HEIGHT: f32 = 0.3;

fn check_collisions(
    time: Res<Time>,
    query: Query<(Entity, &mut PhysicsData, &Collider, &WorldPosition)>,
    collider_query: Query<(Entity, &Collider)>,
) {
    for (entity, mut physics, collider, position) in query {
        if let PhysicsData::Kinematic {
            ref mut displacement,
            ref mut grounded,
            ref mut time_since_grounded,
            ref mut last_grounded_height,
        } = *physics
        {
            apply_gravity(displacement, time.delta_secs());

            let current_position = position.as_vec3();
            let mut detected_ground_collision = false;

            for (other_entity, other_collider) in collider_query {
                if entity == other_entity {
                    continue;
                }

                if let Some(collision) = Collider::check_collision(collider, other_collider) {
                    handle_collision_response(
                        collision,
                        displacement,
                        &mut detected_ground_collision,
                        *grounded,
                        collider,
                        current_position,
                        entity,
                        &collider_query,
                        time.delta_secs(),
                    );
                }
            }

            update_ground_state(
                grounded,
                time_since_grounded,
                last_grounded_height,
                detected_ground_collision,
                current_position.y,
                time.delta_secs(),
            );
        }
    }
}

/// Applies gravity to the character’s current displacement.
fn apply_gravity(displacement: &mut Vec3, delta_time: f32) {
    displacement.y -= GRAVITY * delta_time;
}

/// Handles collision resolution depending on type (ground, wall, step, etc.)
fn handle_collision_response(
    collision: CollisionEvent,
    displacement: &mut Vec3,
    detected_ground_collision: &mut bool,
    grounded: bool,
    collider: &Collider,
    current_position: Vec3,
    entity: Entity,
    query2: &Query<(Entity, &Collider)>,
    delta_time: f32,
) {
    let normal = collision.normal();
    let velocity_along_normal = displacement.dot(normal);

    let is_horizontal = normal.y.abs() < 0.7;

    // Mark as grounded if colliding from below
    if normal.y > 0.7 {
        *detected_ground_collision = true;
    }

    if is_horizontal && velocity_along_normal < 0.0 && grounded {
        // Handle step-up logic for grounded horizontal collisions
        if !try_step_up(
            collider,
            current_position,
            entity,
            query2,
            displacement,
            delta_time,
        ) {
            // Could not step up; cancel velocity into obstacle
            *displacement -= normal * velocity_along_normal;
        }
    } else if velocity_along_normal < 0.0 {
        // Standard vertical or sloped collision resolution
        *displacement -= normal * velocity_along_normal;
    }
}

/// Attempts to "step up" a small ledge if possible.
/// Returns `true` if a valid step-up position was found and applied.
fn try_step_up(
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
fn update_ground_state(
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
    } else {
        *time_since_grounded += delta_time;
    }
}

fn apply_movement(query: Query<(&PhysicsData, &mut WorldPosition)>) {
    for (physics, mut position) in query {
        let new_position = if let PhysicsData::Kinematic { displacement, .. } = *physics {
            position.as_vec3() + displacement
        } else {
            continue;
        };
        position.set(new_position);
        println!("{:?}", new_position);
    }
}
