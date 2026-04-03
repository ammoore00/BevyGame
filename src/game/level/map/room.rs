use crate::Scale;
use crate::game::level::grid;
use crate::game::level::grid::coords::{TileCoords, WorldCoords};
use crate::game::level::grid::tile::assets::TileLayout;
use crate::game::level::grid::tile::tile;
use crate::game::level::grid::TileMap;
use bevy::prelude::*;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use crate::data::{ResourceFileType, ResourceLocation, ResourceType};
use crate::data::loader::{LoaderJobManager, RonAssetLoader};
use crate::data::registry::ResourceRegistry;
use crate::data::sprite::SpriteRegistry;
use crate::datagen_api::tile::codec::{TileAsset, TileRegistry, TileResource};

pub(super) fn plugin(app: &mut App) {
    app.init_asset_loader::<RonAssetLoader<RoomCodec, RoomDefinition>>();
    app.init_asset::<RoomDefinition>();
    app.add_resource_registry::<RoomResource>();
}

type RoomTileCoords = TileCoords;
type RoomWorldCoords = WorldCoords;

#[derive(Serialize, Deserialize)]
pub struct RoomCodec {
    format: u8,
    tile_palette: Vec<ResourceLocation<TileResource>>,
    /// Stored in YZX order (outer to inner)
    tiles: Vec<Vec<Vec<u8>>>
}

/// The type of room this is
/// Set pieces and injectables are rooms designed for a specific instance
/// Transitions are designed to be randomly selected in connector sections from a pool
#[derive(Debug, Reflect)]
pub enum RoomType {
    SetPiece,
    Injectable,
    Transition,
}

/// Elements required to build a room dynamically
#[derive(TypePath, Asset)]
pub struct RoomDefinition {
    /// How this room is intended to be used
    room_type: RoomType,
    /// Connections to other rooms
    connections: Vec<RoomConnection>,
    /// How big this room is
    bounds: UVec3,
    /// Unique ID for this room
    layout: RoomLayout,
}
impl From<RoomCodec> for RoomDefinition {
    fn from(codec: RoomCodec) -> Self {
        let layout = RoomLayout::new(codec.tile_palette, codec.tiles).unwrap();
        
        Self {
            room_type: RoomType::Transition,
            connections: Vec::new(),
            bounds: UVec3::ZERO,
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
        let bounds = layout.bounds();

        Self {
            room_type,
            connections,
            bounds,
            layout,
        }
    }

    pub fn build(
        &self,
        builder_context: &mut RoomBuilderContext,
    ) -> TileMap {
        self.layout.build(builder_context)
    }
}

/// Definition for the connection itself
#[derive(Debug, Clone, Reflect)]
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
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum ConnectionSize {
    Small,
    Medium,
    Large,
}

/// The facing from this room, as seen from the room itself.
///
/// E.g., a North facing exits the current room to the north side
/// and requires a South facing connection to match
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum ConnectionFacing {
    North,
    East,
    South,
    West,
}

pub type RoomRegistry = ResourceRegistry<RoomResource>;

/// Struct which contains the specific tile layout for a room
#[derive(Debug, Clone)]
pub struct RoomLayout {
    bounds: UVec3,
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
    
    fn index_of(&self, coords: impl Into<UVec3>) -> usize {
        let coords = coords.into();
        (coords.x
            + coords.z * self.bounds.x
            + coords.y * self.bounds.x * self.bounds.z) as usize
    }
    
    pub fn build(&self, context: &mut RoomBuilderContext) -> TileMap {
        let tile_map = grid::tile_map();

        for y in 0..self.bounds.y {
            for z in 0..self.bounds.z {
                for x in 0..self.bounds.x {
                    let Some(tile_type) = self.tiles[self.index_of([x, y, z])].clone() else {
                        continue;
                    };

                    let coords = TileCoords(IVec3::new(x as i32, y as i32, z as i32));

                    let tile = context
                        .commands
                        .spawn(tile(
                            context.tile_registry,
                            context.tile_assets,
                            context.sprite_registry,
                            &tile_type,
                            coords.clone(),
                            context.tile_layout,
                        ))
                        .id();

                    tile_map
                        .write()
                        .unwrap()
                        .insert(coords, tile);
                }
            }
        }

        tile_map
    }

    fn bounds(&self) -> UVec3 {
        self.bounds
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RoomLayoutError {
    #[error("Mismatched tile layout sizes")]
    MismatchedSize,
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(u8),
}

/// Context holding references to data necessary to build rooms from their definitions
///
/// RoomBuilderContext is a reference type which is designed to be passed by value
pub struct RoomBuilderContext<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub scale: Scale,
    pub tile_layout: &'a TileLayout,
    pub tile_registry: &'a TileRegistry,
    pub sprite_registry: &'a SpriteRegistry,
    pub tile_assets: &'a Assets<TileAsset>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct RoomResource;
impl ResourceType for RoomResource {
    type AssetType = RoomDefinition;
    
    fn root_dir() -> &'static str {
        "rooms"
    }

    fn file_type() -> ResourceFileType {
        ResourceFileType::Data
    }
}