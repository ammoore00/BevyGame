use crate::Scale;
use crate::game::level::grid::TileMap;
use crate::game::level::grid::coords::{TileCoords, WorldCoords};
use crate::game::level::grid::tile::assets::TileAssets;
use bevy::prelude::*;
use std::fmt::{Debug, Formatter};

pub(super) fn plugin(app: &mut App) {}

type RoomTileCoords = TileCoords;
type RoomWorldCoords = WorldCoords;

#[derive(Debug)]
pub enum RoomType {
    SetPiece,
    Injectable,
    Connector,
}

pub struct RoomDefinition {
    room_type: RoomType,
    connections: Vec<RoomConnection>,
    bounds: IVec3,
    tile_builder: RoomTileBuilder,
}

impl Debug for RoomDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomDefinition")
            .field("room_type", &self.room_type)
            .field("connections", &self.connections)
            .field("bounds", &self.bounds)
            .finish()
    }
}

type RoomTileBuilder = Box<
    dyn FnMut(Commands, Res<Scale>, Res<TileAssets>, ResMut<Assets<TextureAtlasLayout>>) -> TileMap
        + Send
        + Sync,
>;

#[derive(Debug)]
pub struct RoomConnection {
    location: RoomTileCoords,
    connection_type: ConnectionType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    Small,
    Medium,
    Large,
}
