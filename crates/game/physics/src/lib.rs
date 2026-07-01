use bevy::prelude::*;

mod components;
mod math;
mod movement;

// TODO: ColliderType and ToBevy are only used in debug, figure out something to do there to remove them
pub use crate::{
    components::{
        Collider, ColliderType,
        PhysicsData,
    },
    math::ToBevy,
    movement::{
        MovementController,
        DEFAULT_MAX_SPEED,
    },
};


pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((components::plugin, movement::plugin));
    }
}