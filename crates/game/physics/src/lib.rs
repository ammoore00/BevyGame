use bevy::prelude::*;

mod collision;
mod components;
mod math;
mod movement;

pub use crate::{
    components::{Collider, ColliderKind, KinematicData, PhysicsData},
    movement::{DEFAULT_MAX_SPEED, MovementController},
};

#[cfg(feature = "dev")]
pub use crate::{
    components::{CapsuleData, ColliderData},
    math::ToBevy,
};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((collision::plugin, components::plugin, movement::plugin));
    }
}
