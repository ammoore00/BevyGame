use assets::AssetsPlugin;
use bevy::prelude::*;
use common::CommonPlugin;
use physics::PhysicsPlugin;

pub mod characters;
mod level;
mod object;
mod particle;

pub mod prelude {
    pub use crate::{
        characters::{
            attack::AttackHitbox,
            health::Health,
            npc::ai::{
                pathfinding::{
                    pathfinder::Waypoints,
                    strategy::{
                        ReflectPathfindStrategy,
                        follow::{Following, GainedTarget},
                        wander::Wandering,
                    },
                },
                state::{AiState, AiStateKind, transition::SetStateEvent},
            },
            player::Player,
            stamina::Stamina,
        },
        level::grid::{nav::TileNavMap, tile::Tile},
    };
}

pub use crate::level::{LevelLoadedSystems, ResetLevelEvent, SpawnLevelEvent};

pub struct RuntimePlugin;
impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AssetsPlugin,
            CommonPlugin,
            PhysicsPlugin,
            characters::plugin,
            level::plugin,
            particle::plugin,
            object::plugin,
        ));
    }
}
