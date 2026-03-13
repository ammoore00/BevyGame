use bevy::prelude::*;
use rand::Rng;
use crate::game::level::map::palette::Palette;
use crate::game::level::map::room::{RoomBuilderContext, RoomDefinition, RoomRegistryContext};

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

impl MapDefinition {
    /// Creates a usable map state from the definition and provided RNG
    pub fn bake(
        &self,
        rand: impl Rng,
        palette: &Palette,
        room_registry_context: &RoomRegistryContext,
        room_builder_context: RoomBuilderContext,
    ) -> MapState {
        let transition_pool = palette.transition_pool();
        let room = &transition_pool.0[0];

        let grid = room.room().build(room_registry_context, room_builder_context);

        MapState {
            grid,
        }
    }
}

/// Stores the current state of the generated map
#[derive(Component, Debug)]
pub struct MapState {
    grid: Entity,
}

impl MapState {
    pub fn grid(&self) -> Entity {
        self.grid
    }
}

/// Data required to save the generation criteria and state changes to persistent storage
/// for recreation when loading the game
#[derive(Debug)]
pub struct MapPersistence {}