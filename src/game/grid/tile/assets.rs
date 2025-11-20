use bevy::prelude::*;
use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TileAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TileAssets {
    #[dependency]
    grass: Handle<Image>,
    #[dependency]
    dark_planks: Handle<Image>,
    #[dependency]
    light_planks: Handle<Image>,
    #[dependency]
    dark_framed_planks: Handle<Image>,
    #[dependency]
    light_framed_planks: Handle<Image>,
}

impl TileAssets {
    pub(crate) fn get_asset_set_for_material(&self, material: TileMaterial) -> Handle<Image> {
        match material {
            TileMaterial::Grass => self.grass.clone(),
            TileMaterial::DarkPlanks => self.dark_planks.clone(),
            TileMaterial::LightPlanks => self.light_planks.clone(),
            TileMaterial::DarkFramedPlanks => self.dark_framed_planks.clone(),
            TileMaterial::LightFramedPlanks => self.light_framed_planks.clone(),
        }
    }
}

impl FromWorld for TileAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        TileAssets {
            grass: assets.load("images/grass.png"),
            dark_planks: assets.load("images/planks.png"),
            light_planks: assets.load("images/light_planks.png"),
            dark_framed_planks: assets.load("images/framed_planks.png"),
            light_framed_planks: assets.load("images/light_framed_planks.png"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum TileMaterial {
    Grass,
    DarkPlanks,
    LightPlanks,
    DarkFramedPlanks,
    LightFramedPlanks,
}