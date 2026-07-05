use bevy::prelude::*;
use assets::AssetsPlugin;
use common::CommonPlugin;
use physics::PhysicsPlugin;

pub mod character;
mod level;
mod particle;
mod object;

pub mod debug {
    pub use crate::{
        character::{
            npc::ai::pathfinding::{Pathfinder, PathfinderState},
            health::Health,
            player::Player,
            stamina::Stamina,
        },
        level::grid::nav::TileNavMap,
    };
}

pub use crate::{
    level::{LevelLoadedSystems, SpawnLevelEvent, ResetLevelEvent},
};

pub struct RuntimePlugin;
impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AssetsPlugin,
            CommonPlugin,
            PhysicsPlugin,

            character::plugin,
            level::plugin,
            particle::plugin,
            object::plugin,
        ));
    }
}