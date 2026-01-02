use std::collections::HashMap;
use crate::Scale;
use crate::game::level::grid::TileMap;
use crate::game::level::grid::coords::{TileCoords, WorldCoords};
use crate::game::level::grid::tile::assets::TileAssets;
use bevy::prelude::*;
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

#[derive(Asset, Reflect)]
pub struct RoomDefinition {
    room_type: RoomType,
    connections: Vec<RoomConnection>,
    bounds: IVec3,
    id: RoomID,
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
