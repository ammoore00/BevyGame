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

use crate::ApplyForce;
use crate::collision::PhysicsCollisionsProcessedMessage;
use crate::components::{CollisionContact, KinematicData, PhysicsData};
use crate::forces::{AppliedForces, Force, TargetAxes, TargetVelocity};
use crate::states::PhysicsPipeline;
use bevy::prelude::*;
use common::{Facing, WorldCoords, WorldPosition, marker};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (
                apply_impulses,
                apply_controller_intent,
                apply_gravity,
                apply_passive_friction,
                update_facing_from_movement,
            )
                .in_set(PhysicsPipeline::ApplyIntent),
            update_velocity.in_set(PhysicsPipeline::ReactToForces),
            process_collisions.in_set(PhysicsPipeline::RespondToCollisions),
            apply_movement.in_set(PhysicsPipeline::UpdatePositions),
        ),
    );
}

marker!(pub HasGravity);

marker!(pub Gravity);
marker!(pub MovementIntent);
marker!(pub PassiveFriction);

const PASSIVE_FRICTION: f32 = 5.0;

pub const GRAVITY: f32 = 50.0;

pub const MAX_STABLE_SLOPE_ANGLE: f32 = 45.0_f32.to_radians();

pub const DEFAULT_MAX_SPEED: f32 = 2.0;

pub const CONTROLLER_ACCELERATION_FACTOR: f32 = 20.0;

/// These are the movement parameters for our characters's controller.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the characters wants to move in.
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

fn apply_impulses(query: Query<&mut PhysicsData>) {
    for mut physics in query {
        let PhysicsData::Kinematic(ref mut kinematic_data) = *physics else {
            continue;
        };

        while let Some(impulse) = kinematic_data.impulses.pop_front() {
            kinematic_data.next_velocity += impulse.0;
        }
    }
}

fn apply_controller_intent(query: Query<(Entity, &MovementController)>, mut commands: Commands) {
    for (entity, controller) in query {
        let mut intent_velocity = controller.intent * controller.max_speed;

        if controller.sprinting {
            intent_velocity *= Vec3::new(1.5, 1.0, 1.5);
        }

        commands.trigger(ApplyForce::new::<MovementIntent>(
            entity,
            Force::TargetVelocity(TargetVelocity {
                target: intent_velocity,
                should_steer: false,
                acceleration: Some(controller.max_speed * CONTROLLER_ACCELERATION_FACTOR),
                ..Default::default()
            }),
        ));
    }
}

fn apply_gravity(query: Query<Entity, With<HasGravity>>, mut commands: Commands) {
    for entity in query {
        commands.trigger(ApplyForce::new::<Gravity>(
            entity,
            Force::Acceleration(Vec3::NEG_Y * GRAVITY),
        ));
    }
}

fn apply_passive_friction(query: Query<(Entity, &PhysicsData)>, mut commands: Commands) {
    for (entity, physics) in query {
        let PhysicsData::Kinematic(_) = physics else {
            continue;
        };

        commands.trigger(ApplyForce::new::<PassiveFriction>(
            entity,
            Force::TargetVelocity(TargetVelocity {
                target: Vec3::ZERO,
                can_slow: true,
                should_steer: false,
                zero_crossing: false,
                acceleration: Some(PASSIVE_FRICTION),
                axes: TargetAxes::XZ,
            }),
        ));
    }
}

fn update_velocity(
    physics_query: Query<(&mut PhysicsData, &AppliedForces)>,
    forces_query: Query<&Force>,
    time: Res<Time>,
) {
    for (mut physics, forces) in physics_query {
        let PhysicsData::Kinematic(ref mut kinematic_data) = *physics else {
            error!("Forces have been applied to non-kinematic entity!");
            continue;
        };

        let forces = forces_query.iter_many(forces);

        let mut combined_target_velocities = Vec::new();

        for force in forces {
            match *force {
                Force::TargetVelocity(velocity) => {
                    if velocity.should_steer {
                        combined_target_velocities.push(velocity);
                    } else {
                        apply_component_velocity(kinematic_data, velocity, &time);
                    }
                }
                Force::Acceleration(acceleration) => {
                    kinematic_data.next_velocity += acceleration * time.delta_secs();
                }
            }
        }

        steer_target_velocities(kinematic_data, combined_target_velocities, &time);
    }
}

/// Apply per-component velocity
fn apply_component_velocity(
    kinematic_data: &mut KinematicData,
    velocity: TargetVelocity,
    time: &Res<Time>,
) {
    let apply_axis = |current: &mut f32, target: f32| {
        // 1. Instant application if acceleration is None
        let Some(accel_rate) = velocity.acceleration else {
            *current = target;
            return;
        };

        let step = accel_rate * time.delta_secs();

        // 2. Check if moving in the same direction or opposite/stopped
        let same_direction = (*current * target) > 0.0;
        let is_overspeed = current.abs() > target.abs();

        // 3. Overspeed protection check
        if same_direction && is_overspeed && !velocity.can_slow {
            // Player is moving faster than target in the same direction
            // (e.g., knocked forward/boosted) and this force shouldn't slow them down.
            return;
        }

        // 4. Move velocity toward the target value
        let difference = target - *current;

        if difference.abs() <= step {
            // Target reached within this frame
            *current = target;
        } else {
            let next_val = *current + difference.signum() * step;

            // 5. Zero-crossing protection check
            if !velocity.zero_crossing && (*current * next_val) < 0.0 {
                // If crossing zero isn't allowed, stop cleanly at 0.0
                *current = 0.0;
            } else {
                *current = next_val;
            }
        }
    };

    if velocity.axes.x {
        apply_axis(&mut kinematic_data.next_velocity.x, velocity.target.x);
    }
    if velocity.axes.y {
        apply_axis(&mut kinematic_data.next_velocity.y, velocity.target.y);
    }
    if velocity.axes.z {
        apply_axis(&mut kinematic_data.next_velocity.z, velocity.target.z);
    }
}

fn steer_target_velocities(
    kinematic_data: &mut KinematicData,
    targets: Vec<TargetVelocity>,
    time: &Res<Time>,
) {
    if targets.is_empty() {
        return;
    }

    // 1. Accumulate targets and synthesize rules
    let mut composite_target = Vec3::ZERO;
    let mut can_slow = false;
    let mut zero_crossing = false;
    let mut max_accel: Option<f32> = None;

    for t in targets {
        composite_target += t.target;
        can_slow |= t.can_slow;
        zero_crossing |= t.zero_crossing;

        match (max_accel, t.acceleration) {
            (None, Some(a)) => max_accel = Some(a),
            (Some(curr), Some(a)) => max_accel = Some(curr.max(a)),
            _ => {}
        }
    }

    // If all targets are instant (None), apply composite vector immediately
    let Some(accel_rate) = max_accel else {
        kinematic_data.next_velocity = composite_target;
        return;
    };

    let target_speed = composite_target.length();
    if target_speed < 1e-6 {
        // Target is zero: apply standard friction deceleration to zero
        let step = accel_rate * time.delta_secs();
        if kinematic_data.next_velocity.length() <= step {
            kinematic_data.next_velocity = Vec3::ZERO;
        } else {
            kinematic_data.next_velocity -= kinematic_data.next_velocity.normalize() * step;
        }
        return;
    }

    let target_dir = composite_target / target_speed;
    let current_vel = kinematic_data.next_velocity;

    // 2. Project current velocity into Parallel and Perpendicular components
    let parallel_speed = current_vel.dot(target_dir);
    let parallel_vel = target_dir * parallel_speed;
    let perp_vel = current_vel - parallel_vel;

    // 3. Accelerate/Decelerate parallel velocity along the target direction
    let step = accel_rate * time.delta_secs();
    let mut new_parallel_speed = parallel_speed;

    if parallel_speed < target_speed {
        // Moving slower than target: accelerate
        new_parallel_speed = (parallel_speed + step).min(target_speed);
    } else if can_slow {
        // Moving faster than target: decelerate down to target
        new_parallel_speed = (parallel_speed - step).max(target_speed);
    }

    // Handle zero crossing for parallel motion if moving in reverse
    if parallel_speed < 0.0 && !zero_crossing && new_parallel_speed > 0.0 {
        new_parallel_speed = 0.0;
    }

    // 4. Scrub perpendicular drift (lateral friction for tighter steering)
    // Using 1.5x accel_rate gives responsive turns without feeling like a rigid grid
    let perp_friction_step = step * 1.5;
    let perp_len = perp_vel.length();
    let new_perp_vel = if perp_len <= perp_friction_step {
        Vec3::ZERO
    } else {
        perp_vel * ((perp_len - perp_friction_step) / perp_len)
    };

    // Reconstruct velocity
    kinematic_data.next_velocity = (target_dir * new_parallel_speed) + new_perp_vel;
}

fn process_collisions(
    mut message_reader: MessageReader<PhysicsCollisionsProcessedMessage>,
    mut query: Query<(&mut PhysicsData, &WorldPosition)>,
    time: Res<Time>,
) {
    for message in message_reader.read() {
        let Ok((mut physics, pos)) = query.get_mut(message.colliding_entity) else {
            return error!(
                "Failed to get physics data for event entity {:?}",
                message.colliding_entity
            );
        };

        let mut grounded = false;
        let mut ground_normal = Vec3::ZERO;

        let PhysicsData::Kinematic(ref mut kinematic_data) = *physics else {
            return error!("Collision event triggered for non-kinematic physics object!");
        };

        for collision in message.physics_collisions.iter() {
            let collision_response =
                get_collision_response(&collision.contact, kinematic_data, &time);

            kinematic_data.next_velocity += collision_response.velocity_delta;
            if let Some(new_ground_normal) = collision_response.grounded_normal
                && new_ground_normal.y > ground_normal.y
            {
                ground_normal = new_ground_normal;
                grounded = true;
            }
        }

        update_ground_state(kinematic_data, pos.0, grounded, ground_normal, &time);
    }
}

/// Update the facing angle for characters based on their intended movement
fn update_facing_from_movement(query: Query<(&MovementController, &mut Facing)>) {
    for (controller, mut facing) in query {
        let ground_movement = controller.intent.xz();

        if ground_movement.length() > 1e-6 {
            *facing = Facing::from(ground_movement);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CollisionResponse {
    velocity_delta: Vec3,
    grounded_normal: Option<Vec3>,
}

/// Based on the provided collision, determine the response to apply to the displacement
fn get_collision_response(
    collision: &CollisionContact,
    kinematic_data: &KinematicData,
    time: &Res<Time>,
) -> CollisionResponse {
    let normal = collision.normal();
    let grounded_normal = if normal.y > 0.7 { Some(normal) } else { None };

    let velocity_along_normal = kinematic_data.next_velocity.dot(normal);
    let next_displacement = velocity_along_normal * time.delta_secs();

    // If we aren't traveling into the collider at all, then we don't need to offset the displacement
    if velocity_along_normal >= 0.0 {
        return CollisionResponse {
            velocity_delta: Vec3::ZERO,
            grounded_normal,
        };
    }

    let collision_displacement = -normal * next_displacement;

    let velocity_delta = collision_displacement / time.delta_secs();

    CollisionResponse {
        velocity_delta,
        grounded_normal,
    }
}

/// Update information for tracking grounded state in the kinematics state based on collisions
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
        kinematic_data.ground_normal = Some(ground_normal);

        let slope_velocity_adjustment = stabilize_on_slope(ground_normal, time);
        kinematic_data.next_velocity += slope_velocity_adjustment;
    } else {
        kinematic_data.time_since_grounded += time.delta_secs();
        kinematic_data.ground_normal = None;
    }
}

/// Prevent physics objects from sliding down slopes less than the max stable slope angle.
///
/// Returns the adjustment to be made to the velocity to account for slope sliding.
fn stabilize_on_slope(ground_normal: Vec3, time: &Time) -> Vec3 {
    if ground_normal == Vec3::ZERO {
        return Vec3::ZERO;
    }

    // Skip too-steep slopes.
    let slope_angle = ground_normal.angle_between(Vec3::Y);
    if slope_angle > MAX_STABLE_SLOPE_ANGLE {
        return Vec3::ZERO;
    }

    // 1) Compute how much gravity moved us this frame (world velocity caused by gravity).
    //    This must match how you apply gravity in apply_gravity (i.e. GRAVITY * delta_time).
    let gravity_frame = Vec3::new(0.0, -GRAVITY, 0.0) * time.delta_secs();

    // 2) Split that gravity velocity into normal and tangential parts relative to the ground.
    let gravity_normal_comp = ground_normal * gravity_frame.dot(ground_normal);
    let gravity_tangential = gravity_frame - gravity_normal_comp; // this is the downslope vector gravity would cause

    // 3) Subtract THAT tangential gravity contribution from the final velocity.
    //    This removes the passive sliding caused by gravity this frame while preserving
    //    any non-gravity movement (player input, step push).
    let mut velocity = -gravity_tangential;

    // 4) Safety: remove penetration into the surface if any remains.
    let into_surface = velocity.dot(ground_normal);
    if into_surface < 0.0 {
        velocity -= ground_normal * into_surface;
    }

    velocity
}

/// Apply the final physics displacement to the entity's position
fn apply_movement(query: Query<(&PhysicsData, &mut WorldPosition)>, time: Res<Time>) {
    for (physics, mut pos) in query {
        let new_position =
            if let PhysicsData::Kinematic(KinematicData { next_velocity, .. }) = *physics {
                pos.as_vec3() + next_velocity * time.delta_secs()
            } else {
                return;
            };
        pos.set(new_position);
    }
}
