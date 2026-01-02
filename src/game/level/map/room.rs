use crate::Scale;
use crate::game::level::grid::TileMap;
use crate::game::level::grid::coords::{TileCoords, WorldCoords};
use crate::game::level::grid::tile::assets::TileAssets;
use crate::game::level::grid::tile::tile_types::TileType;
use bevy::prelude::*;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RoomIDs>();
    app.init_resource::<RoomBuilderRegistry>();
}

type RoomTileCoords = TileCoords;
type RoomWorldCoords = WorldCoords;

#[derive(Resource, Debug, Default)]
pub struct RoomIDs(usize);
impl RoomIDs {
    pub fn next(&mut self) -> RoomID {
        let next = self.0;
        self.0 += 1;
        next
    }
}
type RoomID = usize;

#[derive(Debug, Reflect)]
pub enum RoomType {
    SetPiece,
    Injectable,
    Connector,
}

#[derive(Asset, Debug, Reflect)]
pub struct RoomDefinition {
    room_type: RoomType,
    connections: Vec<RoomConnection>,
    bounds: IVec3,
    id: RoomID,
}

#[derive(Debug, Reflect)]
pub struct RoomConnection {
    location: RoomTileCoords,
    connection_type: ConnectionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum ConnectionType {
    Small,
    Medium,
    Large,
}

#[derive(Resource, Default)]
pub struct RoomBuilderRegistry {
    room_builders: HashMap<RoomID, RoomTileBuilder>,
}

type RoomTileBuilder = Box<
    dyn FnMut(Commands, Res<Scale>, Res<TileAssets>, ResMut<Assets<TextureAtlasLayout>>) -> TileMap
        + Send
        + Sync,
>;

pub struct RoomLayout<const X: usize, const Y: usize, const Z: usize> {
    pub tiles: [[[TileType; X]; Z]; Y],
}

impl<const X: usize, const Y: usize, const Z: usize> RoomLayout<X, Y, Z> {
    pub const fn new(tiles: [[[TileType; X]; Z]; Y]) -> Self {
        Self { tiles }
    }
}