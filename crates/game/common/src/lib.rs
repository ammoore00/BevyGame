use bevy::prelude::*;

mod coords;
mod app;
mod character;

pub use crate::{
    app::{
        AppSystems,
        Pause, PausableSystems,
    },
    character::Facing,
    coords::{
        TileCoords, TilePosition,
        WorldCoords, WorldPosition,
        ScreenCoords,
        SCREEN_Z_SCALE, TILE_WIDTH, TILE_HEIGHT,
        rotate_screen_space_to_facing,
        rotate_screen_space_to_movement,
    }
};

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            app::plugin,
            coords::plugin,
        ));

        app.insert_resource(Scale(6.0));
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Scale(pub f32);