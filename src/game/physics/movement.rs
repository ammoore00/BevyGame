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
use crate::game::physics::components::{Collider, ColliderType, PhysicsData};
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
    mut query: Query<(Entity, &mut PhysicsData, &Collider, &WorldPosition)>,
    query2: Query<(Entity, &Collider)>,
) {
    query
        .iter_mut()
        .for_each(|(entity, mut physics, collider, position)| {
            if let PhysicsData::Kinematic {
                ref mut displacement,
                ref mut grounded,
                ref mut time_since_grounded,
                ref mut last_grounded_height,
            } = *physics
            {
                // Apply gravity
                displacement.y -= GRAVITY * time.delta_secs();

                let current_position = position.as_vec3();
                let mut detected_ground_collision = false;

                // For each other collision object, check for collisions
                query2
                    .iter()
                    .filter_map(|(other_entity, other_collider)| {
                        if entity == other_entity {
                            None
                        } else {
                            Collider::check_collision(collider, other_collider)
                        }
                    })
                    .for_each(|collision| {
                        let normal = collision.normal();

                        // Project velocity onto collision normal
                        let velocity_along_normal = displacement.dot(normal);

                        // Check if this is a horizontal collision (normal is mostly horizontal)
                        let is_horizontal = normal.y.abs() < 0.7;

                        // Check if we're on the ground (collision from below)
                        if normal.y > 0.7 {
                            detected_ground_collision = true;
                        }

                        if is_horizontal && velocity_along_normal < 0.0 && *grounded {
                            // This is a horizontal collision where we're moving into the obstacle while grounded
                            // Try to step up by checking collision at a higher position
                            let mut found_step_height = false;

                            for test_height in 1..=10 {
                                let test_step = (test_height as f32) * (STEP_UP_HEIGHT / 10.0);
                                let test_position = current_position + Vec3::Y * test_step;

                                // Create a test collider at the elevated position
                                let test_collider = Collider::with_collider(collider.collider_type().clone(), test_position);

                                // Check if there's still a collision at this height
                                let still_colliding = query2
                                    .iter()
                                    .filter(|(other_entity, _)| *other_entity != entity)
                                    .any(|(_, other_collider)| {
                                        test_collider.check_collision(other_collider).is_some()
                                    });

                                if !still_colliding {
                                    // Found a height where we can pass!
                                    // Set a small upward velocity to smoothly lift the character
                                    displacement.y =
                                        displacement.y.max(test_step * time.delta_secs()); // Multiply to convert to velocity
                                    found_step_height = true;
                                    break;
                                }
                            }

                            // If we couldn't find a step-up height, block horizontal movement
                            if !found_step_height {
                                *displacement -= normal * velocity_along_normal;
                            }
                        } else if velocity_along_normal < 0.0 {
                            // This is a vertical or other collision
                            *displacement -= normal * velocity_along_normal;
                        }
                    });

                *grounded = detected_ground_collision;

                if *grounded {
                    *time_since_grounded = 0.0;
                    *last_grounded_height = current_position.y;
                } else {
                    *time_since_grounded += time.delta_secs();
                }
            }
        });
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