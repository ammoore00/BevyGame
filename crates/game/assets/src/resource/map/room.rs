use crate::codec::RoomCodec;
use crate::loader::{LoaderJobManager, RonAssetLoader};
use crate::resource::TileResource;
use bevy::prelude::*;
use common::TileCoords;
use data::define_data_resource;
use data::loc::ResourceLocation;
use getset::Getters;
use serde::{Deserialize, Serialize};

pub(super) fn plugin(app: &mut App) {
    app.init_asset_loader::<RonAssetLoader<RoomCodec, RoomDefinition>>();
    app.init_asset::<RoomDefinition>();
    app.add_registry_with_discovery::<RoomResource>();
}

define_data_resource!(Room, "rooms", RoomDefinition);

type RoomTileCoords = TileCoords;

/// The type of room this is
/// Set pieces and injectables are rooms designed for a specific instance
/// Transitions are designed to be randomly selected in connector sections from a pool
#[derive(Debug, Clone, Copy)]
pub enum RoomType {
    SetPiece,
    Injectable,
    Transition,
}

/// Elements required to build a room dynamically
#[derive(Asset, Debug, Clone, Getters, TypePath)]
pub struct RoomDefinition {
    /// How this room is intended to be used
    _room_type: RoomType,
    /// Connections to other rooms
    _connections: Vec<RoomConnection>,
    /// How big this room is
    _bounds: UVec3,
    /// Unique ID for this room
    #[getset(get = "pub")]
    layout: RoomLayout,
}
impl From<RoomCodec> for RoomDefinition {
    fn from(codec: RoomCodec) -> Self {
        let layout = RoomLayout::new(codec.tile_palette, codec.tiles).unwrap();

        Self {
            _room_type: RoomType::Transition,
            _connections: codec.connections,
            _bounds: UVec3::ZERO,
            layout,
        }
    }
}
impl RoomDefinition {
    pub fn new(
        room_type: RoomType,
        connections: Vec<RoomConnection>,
        layout: RoomLayout,
    ) -> Self {
        let bounds = layout.bounds().clone();

        Self {
            _room_type: room_type,
            _connections: connections,
            _bounds: bounds,
            layout,
        }
    }
}

/// Definition for the connection itself
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct RoomConnection {
    /// Where in the room this connection is located
    location: RoomTileCoords,
    /// How big this connection is
    connection_size: ConnectionSize,
    /// The facing of this connection, as seen from the room itself
    facing: ConnectionFacing,
}

impl RoomConnection {
    pub fn new(location: RoomTileCoords, connection_size: ConnectionSize, facing: ConnectionFacing) -> Self {
        Self { location, connection_size, facing }
    }
}

/// The type of connection this is
/// Standardized connection sizes are used to allow for flexibility when matching rooms dynamically
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum ConnectionSize {
    Small,
    Medium,
    Large,
}

/// The facing from this room, as seen from the room itself.
///
/// E.g., a North facing exits the current room to the north side
/// and requires a South facing connection to match
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum ConnectionFacing {
    North,
    East,
    South,
    West,
}

/// Struct which contains the specific tile layout for a room
#[derive(Debug, Clone, Getters)]
pub struct RoomLayout {
    #[getset(get = "pub")]
    bounds: UVec3,
    #[getset(get = "pub")]
    tiles: Vec<Option<ResourceLocation<TileResource>>>,
}
impl RoomLayout {
    pub fn new(
        tile_palette: Vec<ResourceLocation<TileResource>>,
        tiles: Vec<Vec<Vec<u8>>>
    ) -> Result<Self, RoomLayoutError> {
        let bounds = UVec3::new(tiles[0][0].len() as u32, tiles.len() as u32, tiles[0].len() as u32);

        if tiles.iter().any(|yz| yz.len() != bounds.z as usize
            || yz.iter().any(|xyz| xyz.len() != bounds.x as usize))
        {
            return Err(RoomLayoutError::MismatchedSize);
        }

        let tiles = tiles
            .into_iter()
            .flat_map(|yz| yz.into_iter())
            .flat_map(|x| x.into_iter())
            .map(|index: u8| {
                match index {
                    0 => None,
                    _ => {
                        if index as usize > tile_palette.len() {
                            return None;
                        }
                        Some(tile_palette[index as usize - 1].clone())
                    },
                }
            })
            .collect();

        Ok(Self {
            bounds,
            tiles
        })
    }

    pub fn index_of(&self, coords: impl Into<UVec3>) -> usize {
        let coords = coords.into();
        (coords.x
            + coords.z * self.bounds.x
            + coords.y * self.bounds.x * self.bounds.z) as usize
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RoomLayoutError {
    #[error("Mismatched tile layout sizes")]
    MismatchedSize,
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(u8),
}