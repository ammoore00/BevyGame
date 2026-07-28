use crate::resource::level::palette::Palette;
use bevy::prelude::*;
use data::define_data_resource;
use data::prelude::ResourceLocation;
use getset::Getters;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<MapDefinition>();
}

define_data_resource!(Map, "level/maps", MapDefinition);

/// Pool of all registered level definitions
/// This contains every level def that the game knows about
#[derive(Debug, Clone, Default)]
pub struct MapPool(pub Vec<MapDefinition>);

/// The type for this level. This affects where in the world it will be used
///
/// Main - used as a main level for a world
/// Boss - boss dungeon, which may or may not be optional, depending on where it gets used
/// Side - optional side dungeon without a boss
#[derive(Debug, Clone, Copy)]
pub enum MapType {
    Main,
    _Boss,
    _Side,
}

/// Contains all the information needed to generate a level
///
/// This contains both information about level selection and information about building the level
/// - Selection information includes things such as level type and palette
/// - Build information includes things like set pieces, injectables, and connections
#[derive(Asset, Debug, Clone, Getters, derive_new::new, TypePath)]
pub struct MapDefinition {
    #[getset(get = "pub")]
    _map_type: MapType,

    // Temporary
    #[getset(get = "pub")]
    map_size: usize,
}
impl MapDefinition {
    pub const PLACEHOLDER: Self = Self {
        _map_type: MapType::Main,
        map_size: 1,
    };
}

// TODO: Replace this with actually loading level definitions from data files
#[derive(Component, Default, Clone)]
pub struct MapDataLocation(pub Option<MapDefinition>, pub Option<Palette>);
impl From<MapDataLocation> for ResourceLocation<MapResource> {
    fn from(_value: MapDataLocation) -> Self {
        todo!()
    }
}
