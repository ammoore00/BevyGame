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
    app.init_resource::<RoomIDs>();
    app.init_resource::<RoomBuilderRegistry>();
}

type RoomTileCoords = TileCoords;
type RoomWorldCoords = WorldCoords;

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
    bounds: UVec3,
    id: RoomID,
}

impl RoomDefinition {
    pub fn new(
        room_type: RoomType,
        connections: Vec<RoomConnection>,
        layout: Box<dyn RoomBuilder>,
        registry_context: RoomRegistryContext,
    ) -> Self {
        let bounds = layout.bounds();

        let id = registry_context.ids.next();
        registry_context.registry.room_builders.insert(id, layout);

        Self {
            room_type,
            connections,
            bounds,
            id,
        }
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

#[derive(Resource, Debug, Default)]
pub struct RoomIDs(RoomID);
impl RoomIDs {
    pub fn next(&mut self) -> RoomID {
        let next = self.0;
        self.0 += 1;
        next
    }
}
type RoomID = usize;

#[derive(Resource, Default)]
pub struct RoomBuilderRegistry {
    room_builders: HashMap<RoomID, Box<dyn RoomBuilder>>,
}

pub trait RoomBuilder: Send + Sync {
    fn build(&self, context: RoomBuilderContext) -> Entity;

    fn bounds(&self) -> UVec3;
}

#[derive(Debug, Clone)]
pub struct RoomLayout<const X: usize, const Y: usize, const Z: usize> {
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

pub struct RoomBuilderContext<'a> {
    commands: &'a mut Commands<'a, 'a>,
    scale: Scale,
    tile_assets: &'a TileAssets,
}

pub struct RoomRegistryContext<'a> {
    ids: &'a mut RoomIDs,
    registry: &'a mut RoomBuilderRegistry,
}
