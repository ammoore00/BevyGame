use bevy::prelude::*;

mod collision;
mod components;
mod forces;
mod math;
mod movement;
mod states;

pub use crate::{
    collision::{DetectorCollision, DetectorCollisionsProcessedMessage},
    components::{Collider, ColliderKind, KinematicData, PhysicsData},
    forces::{ApplyForce, ApplyImpulse, Impulse, RemoveForce},
    movement::{DEFAULT_MAX_SPEED, MovementController},
    states::DetectorCollisionResponse,
};

#[cfg(feature = "dev")]
pub use crate::{
    components::{CapsuleData, ColliderData},
    math::ToBevy,
};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            collision::plugin,
            components::plugin,
            forces::plugin,
            movement::plugin,
            states::plugin,
        ));
    }
}
