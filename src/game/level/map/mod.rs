use crate::data::prelude::*;
use crate::game::level::grid;
use crate::game::level::grid::tile::{set_tile_location, TileEntity};
use crate::game::level::grid::{grid_bundle, merge_tile_map, Grid};
use crate::game::level::map::palette::Palette;
use crate::game::level::map::room::RoomBuilderContext;
use crate::{data, register_prototype_system};
use bevy::ecs::query::{QueryData, QueryItem};
use bevy::prelude::*;
use getset::CopyGetters;
use rand::{Rng, RngExt};
use crate::game::level::grid::nav::{NavContext, TileNavMap};

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

    app.add_systems(
        Update,
        initialize_maps
    );
}

define_data_resource!(Map, "level/maps", MapDefinition);
register_prototype_system!(initialize_maps, MapBuilder);

/// Pool of all registered map definitions
/// This contains every map def that the game knows about
#[derive(Debug, Clone, Default)]
pub struct MapPool(pub(crate) Vec<MapDefinition>);

/// The type for this map. This affects where in the world it will be used
///
/// Main - used as a main map for a world
/// Boss - boss dungeon, which may or may not be optional, depending on where it gets used
/// Side - optional side dungeon without a boss
#[derive(Debug, Clone, Copy)]
pub enum MapType {
    Main,
    _Boss,
    _Side,
}

/// Contains all the information needed to generate a map
///
/// This contains both information about map selection and information about building the map
/// - Selection information includes things such as map type and palette
/// - Build information includes things like set pieces, injectables, and connections
#[derive(Asset, Debug, Clone, TypePath)]
pub struct MapDefinition {
    map_type: MapType,

    // Temporary
    map_size: usize,
}
impl MapDefinition {
    const PLACEHOLDER: Self = Self {
        map_type: MapType::Main,
        map_size: 1,
    };
}

// TODO: Replace this with actually loading map definitions from data files
#[derive(Component, Default, Clone)]
pub struct MapDataLocation(Option<MapDefinition>, Option<Palette>);
impl From<MapDataLocation> for ResourceLocation<MapResource> {
    fn from(_value: MapDataLocation) -> Self {
        todo!()
    }
}

#[derive(Clone)]
pub struct MapProps {
    definition: MapDefinition,
    palette: Palette,
}
impl Default for MapProps {
    fn default() -> Self {
        Self {
            definition: MapDefinition::PLACEHOLDER,
            palette: Palette::default(),
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct Map(());
impl PrototypeFinalizedMarker for Map {
    fn new(_: PrototypeMarkerToken) -> Self { Self(()) }
}

#[derive(SceneComponent, Default, Clone)]
#[scene(MapProps)]
pub struct MapPrototype;
impl MapPrototype {
    pub fn scene(props: MapProps) -> impl Scene {
        bsn! [
            Map
            Transform
            Visibility
            MapDataLocation({Some(props.definition)}, {Some(props.palette)})
        ]
    }
}
impl Prototype for MapPrototype {
    type Marker = Map;
    type Resource = MapResource;
    type DataLocation = MapDataLocation;
}

struct MapBuilder;
impl PrototypeBuilder for MapBuilder {
    type Proto = MapPrototype;
    type Context<'w, 's> = RoomBuilderContext<'w, 's>;
    type QueryData<'w, 's> = ();

    fn build(
        entity: Entity,
        loc: &<Self::Proto as Prototype>::DataLocation,
        _: &QueryItem<'_, '_, <Self::QueryData<'_, '_> as QueryData>::ReadOnly>,
        context: &mut Self::Context<'_, '_>,
        _: Commands
    ) -> Result<(), BevyError> {
        // TODO: Real data handling
        let data = loc.0.as_ref().unwrap();
        let palette = loc.1.as_ref().unwrap();

        // TODO: Propagate errors up
        let grid_entity = spawn_map_grid(data, rand::rng(), palette, context);
        context.commands.entity(entity).add_child(grid_entity);

        Ok(())
    }
}

pub fn map_scene(definition: &MapDefinition, palette: &Palette) -> impl Scene {
    bsn! [
        @MapPrototype {
            @definition: {definition.clone()},
            @palette: {palette.clone()},
        }
    ]
}

pub fn spawn_map_grid(
    // TODO: Map definitions should be loaded from Palette?
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

pub fn bake_nav(
    context: NavContext,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let (map_entity, children) = context.map.single()
        .map_err(|err| {
            BevyError::error(format!("Error getting level entity: {:?}", err))
        })?;

    let mut tile_map = None;
    for child in children {
        if let Ok(grid) = context.grid.get(*child) {
            tile_map = Some(grid.tile_map().clone());
            break;
        }
    }
    let tile_map = tile_map.ok_or(BevyError::error("No grid child found for level map"))?;

    let tile_nav_map = TileNavMap::from_map(tile_map, context.nav_query);
    let nav_entity = commands.spawn(tile_nav_map).id();
    commands.entity(map_entity).add_child(nav_entity);
    
    Ok(())
}

/// Stores the current state of the generated map
#[derive(Component, Debug, CopyGetters)]
pub struct _MapState {
    #[getset(get_copy = "pub")]
    grid: Entity,
    #[getset(get_copy = "pub")]
    nav: Entity,
}

/// Data required to save the generation criteria and state changes to persistent storage
/// for recreation when loading the game
#[derive(Debug)]
pub struct _MapPersistence {}