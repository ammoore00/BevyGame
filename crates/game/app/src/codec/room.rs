use crate::data::prelude::*;
use crate::game::level::map::room::RoomConnection;
use bevy::prelude::TypePath;
use data::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, TypePath)]
pub struct RoomCodec {
    pub format: u8,
    pub tile_palette: Vec<ResourceLocation<TileResource>>,
    /// Stored in YZX order (outer to inner)
    pub tiles: Vec<Vec<Vec<u8>>>,
    pub connections: Vec<RoomConnection>,
}

impl RoomCodec {
    pub fn new(
        format: u8,
        tile_palette: Vec<ResourceLocation<TileResource>>,
        tiles: Vec<Vec<Vec<u8>>>,
        connections: Vec<RoomConnection>,
    ) -> Self {
        Self {
            format,
            tile_palette,
            tiles,
            connections,
        }
    }
}