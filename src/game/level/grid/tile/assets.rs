use std::str::FromStr;
use crate::asset_tracking::LoadResource;
use bevy::prelude::*;
use crate::data::ResourceLocation;
use crate::data::sprite::SpriteRegistry;
use crate::StartupSystems;

pub(in crate::game) fn plugin(app: &mut App) {
    app.add_systems(Startup, register_tile_assets.in_set(StartupSystems::RegisterManifests));
    
    app.load_resource::<TileAssets>();
}

pub fn register_tile_assets(
    mut sprite_registry: ResMut<SpriteRegistry>
) {
    let msg = "Invalid resource location";
    
    sprite_registry.insert_manifest(ResourceLocation::from_str("grass").expect(msg));
    sprite_registry.insert_manifest(ResourceLocation::from_str("planks").expect(msg));
    sprite_registry.insert_manifest(ResourceLocation::from_str("light_planks").expect(msg));
    sprite_registry.insert_manifest(ResourceLocation::from_str("framed_planks").expect(msg));
    sprite_registry.insert_manifest(ResourceLocation::from_str("light_framed_planks").expect(msg));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TileAssets {
    //#[dependency]
    layout: Handle<TextureAtlasLayout>,
    
    #[dependency]
    grass_sprite: Handle<Image>,
    #[dependency]
    dark_planks_sprite: Handle<Image>,
    #[dependency]
    light_planks_sprite: Handle<Image>,
    #[dependency]
    dark_framed_planks_sprite: Handle<Image>,
    #[dependency]
    light_framed_planks_sprite: Handle<Image>,
}

impl TileAssets {
    pub fn get_asset_set_for_material(&self, material: TileMaterial) -> Handle<Image> {
        match material {
            TileMaterial::Grass => self.grass_sprite.clone(),
            TileMaterial::DarkPlanks => self.dark_planks_sprite.clone(),
            TileMaterial::LightPlanks => self.light_planks_sprite.clone(),
            TileMaterial::DarkFramedPlanks => self.dark_framed_planks_sprite.clone(),
            TileMaterial::LightFramedPlanks => self.light_framed_planks_sprite.clone(),
        }
    }
    
    pub fn layout(&self) -> Handle<TextureAtlasLayout> {
        self.layout.clone()
    }
}

impl FromWorld for TileAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 8, Some(UVec2::splat(1)), None);
        
        TileAssets {
            layout: assets.add(layout),
            
            grass_sprite: assets.load("images/grass.png"),
            dark_planks_sprite: assets.load("images/planks.png"),
            light_planks_sprite: assets.load("images/light_planks.png"),
            dark_framed_planks_sprite: assets.load("images/framed_planks.png"),
            light_framed_planks_sprite: assets.load("images/light_framed_planks.png"),
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
