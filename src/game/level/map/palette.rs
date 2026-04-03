use bevy::prelude::*;
use crate::game::level::map::transition::{TransitionRoom, TransitionRoomPool};
use crate::game::level::map::{MapDefinition, MapPool, MapType};

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<Palette>();
    app.init_resource::<Palettes>();
}

/// Resource holding handles to each palette asset
#[derive(Resource, Asset, Debug, Reflect)]
pub struct Palettes {
    pub standard: Handle<Palette>,
}

impl FromWorld for Palettes {
    fn from_world(world: &mut World) -> Self {
        let standard_palette = StandardPalette::create_palette();

        let assets = world.resource::<AssetServer>();

        Self {
            standard: assets.add(standard_palette),
        }
    }
}

#[derive(Asset, TypePath)]
pub struct Palette {
    main_map_pool: MapPool,
    transition_pool: TransitionRoomPool,
}

impl Palette {
    pub fn transition_pool(&self) -> &TransitionRoomPool {
        &self.transition_pool
    }
    
    pub fn main_map_pool(&self) -> &MapPool {
        &self.main_map_pool
    }
}

trait PaletteDefinition {
    fn create_palette() -> Palette;
    fn create_main_map_pool() -> MapPool;
    fn create_transition_pool() -> TransitionRoomPool;
}

struct StandardPalette;
impl PaletteDefinition for StandardPalette {
    fn create_palette() -> Palette {
        Palette {
            main_map_pool: Self::create_main_map_pool(),
            transition_pool: Self::create_transition_pool(),
        }
    }

    fn create_main_map_pool() -> MapPool {
        let main_map = MapDefinition {
            map_type: MapType::Main,
            map_size: 3,
        };
        MapPool(vec![main_map])
    }

    fn create_transition_pool() -> TransitionRoomPool {
        TransitionRoomPool(vec![
            TransitionRoom::new("basic_grass".parse().unwrap(), 1.0),
            TransitionRoom::new("basic_planks".parse().unwrap(), 1.0),
        ])
    }
}