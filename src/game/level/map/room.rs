use crate::Scale;
use crate::game::level::grid;
use crate::game::level::grid::coords::{TileCoords, WorldCoords};
use crate::game::level::grid::tile::assets::TileAssets;
use crate::game::level::grid::tile::tile;
use crate::game::level::grid::tile::tile_types::TileType;
use crate::game::level::grid::grid;
use bevy::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RoomRegistryContext>();
}

type RoomTileCoords = TileCoords;
type RoomWorldCoords = WorldCoords;

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
#[derive(Debug, Reflect)]
pub struct RoomDefinition {
    /// How this room is intended to be used
    room_type: RoomType,
    /// Connections to other rooms
    connections: Vec<RoomConnection>,
    /// How big this room is
    bounds: UVec3,
    /// Unique ID for this room
    id: RoomID,
}

impl RoomDefinition {
    pub fn new(
        room_type: RoomType,
        connections: Vec<RoomConnection>,
        layout: Box<dyn RoomBuilder>,
        registry_context: &mut RoomRegistryContext,
    ) -> Self {
        let bounds = layout.bounds();

        let id = registry_context.ids.next_id();
        registry_context.registry.room_builders.insert(id, layout);

        Self {
            room_type,
            connections,
            bounds,
            id,
        }
    }

    pub fn build(
        &self,
        registry_context: &RoomRegistryContext,
        builder_context: RoomBuilderContext,
    ) -> Entity {
        registry_context.registry.room_builders[&self.id].build(builder_context)
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

/// Context holding references to data necessary to register rooms globally
#[derive(Resource, Default)]
pub struct RoomRegistryContext {
    ids: RoomIDTracker,
    registry: RoomBuilderRegistry,
}

/// Manager for ensuring unique identifiers for rooms
/// These ids do not need to be the same every time, they only need to be unique from other rooms
#[derive(Debug, Default)]
pub struct RoomIDTracker(RoomID);
impl RoomIDTracker {
    pub fn next_id(&mut self) -> RoomID {
        let next = self.0;
        self.0 += 1;
        next
    }
}
type RoomID = usize;

/// Registry which holds references from each room id to the builder which can generate the room
/// from the definition
///
/// This is kept separate from the room definitions to allow for reflection in the room definitions
/// while keeping the room builders generic
#[derive(Default)]
pub struct RoomBuilderRegistry {
    room_builders: HashMap<RoomID, Box<dyn RoomBuilder>>,
}

/// Trait for building rooms from a layout
///
/// This is implemented as a trait to allow for const generic room layouts
pub trait RoomBuilder: Send + Sync {
    fn build(&self, context: RoomBuilderContext) -> Entity;

    fn bounds(&self) -> UVec3;
}

/// Struct which contains the specific tile layout for a room
#[derive(Debug, Clone)]
pub struct RoomLayout<
    const X: usize,
    const Y: usize,
    const Z: usize
> {
    tiles: [[[Option<TileType>; X]; Z]; Y],
}

impl<const X: usize, const Y: usize, const Z: usize> RoomLayout<X, Y, Z> {
    pub const fn new(tiles: [[[Option<TileType>; X]; Z]; Y]) -> Self {
        Self { tiles }
    }
}

impl<const X: usize, const Y: usize, const Z: usize> RoomBuilder for RoomLayout<X, Y, Z> {
    fn build(&self, context: RoomBuilderContext) -> Entity {
        let tile_map = grid::tile_map();
        let grid = grid(tile_map.clone(), context.scale.0);
        let grid = context.commands.spawn(grid).id();

        for y in 0..Y {
            for z in 0..Z {
                for x in 0..X {
                    let Some(tile_type) = self.tiles[y][z][x] else {
                        continue;
                    };

                    let coords = TileCoords(IVec3::new(x as i32, y as i32, z as i32));

                    let tile = context
                        .commands
                        .spawn(tile(
                            tile_type,
                            coords.clone(),
                            context.tile_assets,
                        ))
                        .id();

                    context.commands.entity(grid).add_child(tile);
                    tile_map
                        .write()
                        .unwrap()
                        .insert(coords, tile);
                }
            }
        }

        grid
    }

    fn bounds(&self) -> UVec3 {
        UVec3::new(X as u32, Y as u32, Z as u32)
    }
}

/// Context holding references to data necessary to build rooms from their definitions
///
/// RoomBuilderContext is a reference type which is designed to be passed by value
pub struct RoomBuilderContext<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub scale: Scale,
    pub tile_assets: &'a TileAssets,
}
