use bevy::prelude::*;

mod app;
mod character;
mod coords;
mod game_states;
pub mod macros;

#[cfg(feature = "dev")]
pub mod dev_tools;

pub use crate::{
    app::{AppSystems, GameInputSystems, InputBlocker, PausableSystems, Pause},
    character::{Facing, offset_position_to_facing},
    coords::{
        SCREEN_Z_SCALE, ScreenCoords, TILE_HEIGHT, TILE_WIDTH, TileCoords, TilePosition,
        WorldCoords, WorldPosition, convert_world_to_screen_coords, rotate_screen_space_to_facing,
        rotate_screen_space_to_movement,
    },
    game_states::{GameState, GameplaySystems},
};

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((app::plugin, coords::plugin, game_states::plugin));

        app.insert_resource(Scale(3.0));
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Scale(pub f32);
