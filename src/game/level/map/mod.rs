use crate::Scale;
use crate::game::level::grid::coords::TileCoords;
use crate::game::level::grid::{grid, tile_map};
use crate::game::level::grid::tile::assets::TileAssets;
use crate::game::level::grid::tile::tile;
use crate::game::level::grid::tile::tile_types::TileType;
use bevy::prelude::*;
use rand::Rng;
use crate::game::level::map::palette::Palette;
use crate::game::level::map::room::{RoomBuilderContext, RoomDefinition};

pub mod palette;
pub mod room;
mod connector;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<MapDefinition>();

    app.add_plugins((
        connector::plugin,
        room::plugin,
        palette::plugin, // Order matters here - palette must be after room
    ));
}

#[derive(Debug, Reflect)]
pub struct MapPool(pub(crate) Vec<MapDefinition>);

#[derive(Debug, Reflect)]
pub enum MapType {
    Main,
    Boss,
    Side,
}

#[derive(Asset, Debug, Reflect)]
pub struct MapDefinition {
    map_type: MapType,

    // Temporary
    map_size: usize,
}

impl MapDefinition {
    pub fn bake(
        &self,
        rand: impl Rng,
        palette: &Palette,
        room_builder_context: RoomBuilderContext
    ) -> MapState {
        todo!()
    }
}

#[derive(Component, Debug)]
pub struct MapState {
    grid: Entity,
    injectable: RoomDefinition,
}

pub fn map_state(
    grid: Entity,
    injectable: RoomDefinition,
) -> impl Bundle {
    MapState {
        grid,
        injectable,
    }
}

#[derive(Debug)]
pub struct MapPersistence {}