use bevy::prelude::*;

mod map;
mod palette;
mod room;
pub mod transition;

pub use {
    map::{MapDataLocation, MapDefinition, MapRegistry, MapResource},
    palette::{Palette, Palettes},
    room::{ConnectionSize, ConnectionFacing, RoomConnection, RoomDefinition, RoomLayout, RoomRegistry, RoomResource},
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        map::plugin,
        palette::plugin,
        room::plugin,
        transition::plugin,
    ));
}
