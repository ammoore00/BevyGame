use crate::{WriteError, write_data};
use assets::codec::RoomCodec;
use assets::resource::level::{RoomConnection, RoomResource, TileResource};
use data::prelude::*;

pub fn create_room_data(room_data: RoomData) -> Result<(), WriteError> {
    let loc = room_data.loc.clone();
    let codec = RoomCodec::from(room_data);
    write_data(loc, &codec)
}

pub struct RoomData {
    loc: ResourceLocation<RoomResource>,
    tile_palette: Vec<ResourceLocation<TileResource>>,
    /// Stored in YZX order (outer to inner)
    tiles: Vec<Vec<Vec<u8>>>,
    connections: Vec<RoomConnection>,
}
impl RoomData {
    pub fn new(
        loc: &str,
        tile_palette: Vec<ResourceLocation<TileResource>>,
        tiles: Vec<Vec<Vec<u8>>>,
        connections: Vec<RoomConnection>,
    ) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            tile_palette,
            tiles,
            connections,
        }
    }
}
impl From<RoomData> for RoomCodec {
    fn from(data: RoomData) -> Self {
        RoomCodec::new(
            LATEST_FORMAT,
            data.tile_palette,
            data.tiles,
            data.connections,
        )
    }
}

const LATEST_FORMAT: u8 = 1;
