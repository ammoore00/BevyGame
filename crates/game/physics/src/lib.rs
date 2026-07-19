use bevy::prelude::*;

mod components;
mod math;
mod movement;

pub use crate::{
    components::{Collider, PhysicsData},
    movement::{DEFAULT_MAX_SPEED, MovementController},
};

#[cfg(feature = "dev")]
pub use crate::{
    components::{CapsuleData, ColliderType},
    math::ToBevy,
};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((components::plugin, movement::plugin));
    }
}
