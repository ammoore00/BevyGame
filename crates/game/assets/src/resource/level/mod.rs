use bevy::prelude::*;

mod map;
mod palette;
mod room;
mod tile;
mod transition;

pub use {
    map::{MapDataLocation, MapDefinition, MapRegistry, MapResource},
    palette::{Palette, Palettes},
    room::{
        ConnectionFacing, ConnectionSize, RoomConnection, RoomDefinition, RoomLayout, RoomRegistry,
        RoomResource,
    },
    tile::{
        TileAsset, TileFacing, TileLayout, TileRegistry, TileResource, TileShape,
        TileSpriteRegistry, TileSpriteResource,
    },
    transition::{TransitionRoom, TransitionRoomPool},
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        map::plugin,
        palette::plugin,
        room::plugin,
        tile::plugin,
        transition::plugin,
    ));
}
