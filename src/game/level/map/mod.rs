use crate::game::level::grid;
use crate::game::level::grid::tile::{set_tile_location, TileEntity};
use crate::game::level::grid::{grid_bundle, merge_tile_map, Grid};
use crate::game::level::map::palette::Palette;
use crate::game::level::map::room::RoomBuilderContext;
use bevy::prelude::*;
use rand::{Rng, RngExt};

pub mod palette;
pub mod room;
mod transition;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<MapDefinition>();

    app.add_plugins((
        transition::plugin,
        room::plugin,
        palette::plugin, // Order matters here - palette must be after room
    ));
}

/// Pool of all registered map definitions
/// This contains every map def that the game knows about
#[derive(Debug, Reflect)]
pub struct MapPool(pub(crate) Vec<MapDefinition>);

/// The type for this map. This affects where in the world it will be used
///
/// Main - used as a main map for a world
/// Boss - boss dungeon, which may or may not be optional, depending on where it gets used
/// Side - optional side dungeon without a boss
#[derive(Debug, Reflect)]
pub enum MapType {
    Main,
    Boss,
    Side,
}

/// Contains all the information needed to generate a map
///
/// This contains both information about map selection and information about building the map
/// - Selection information includes things such as map type and palette
/// - Build information includes things like set pieces, injectables, and connections
#[derive(Asset, Debug, Reflect)]
pub struct MapDefinition {
    map_type: MapType,

    // Temporary
    map_size: usize,
}

pub fn map_bundle() -> impl Bundle {
    (
        Map,
        Transform::default(),
        Visibility::default(),
    )
}

pub fn build_map_grid(
    map_definition: &MapDefinition,
    mut rand: impl Rng,
    palette: &Palette,
    context: &mut RoomBuilderContext,
) -> Entity {
    let transition_pool = palette.transition_pool();

    let mut grid = Grid::new(grid::tile_map());

    for _ in 0..map_definition.map_size {
        let index = rand.random_range(0..transition_pool.0.len());
        let room = transition_pool.0[index].room();

        let room = context.room_registry.get(room).unwrap();
        let room = context.room_assets.get(room).cloned().unwrap();

        let room_tile_map = room.build(context);

        let grid_size = grid.size();
        merge_tile_map(grid.tile_map_mut(), room_tile_map, IVec3::new(grid_size.x as i32, 0, 0))
            .expect("Failed to merge tile map");
    }

    let grid_entity = context.commands
        .spawn(grid_bundle(grid.clone(), context.scale.0))
        .id();

    for (tile_coords, tile) in &*grid.tile_map().read().unwrap() {
        context.commands.entity(grid_entity).add_child(*tile);

        let tile = TileEntity(context.commands.entity(*tile).id());
        set_tile_location(tile, tile_coords.clone(), &mut context.commands);
    }

    grid_entity
}

#[derive(Component, Debug, Clone)]
pub struct Map;

/// Stores the current state of the generated map
#[derive(Component, Debug)]
pub struct MapState {
    grid: Entity,
    nav: Entity,
}

impl MapState {
    pub fn grid(&self) -> Entity {
        self.grid
    }
}

/// Data required to save the generation criteria and state changes to persistent storage
/// for recreation when loading the game
#[derive(Debug)]
pub struct _MapPersistence {}