use std::sync::LazyLock;
use bevy::prelude::*;
use crate::data::ResourceLocation;
use crate::data::sprite::{SpriteRegistry, SpriteResource};
use crate::StartupSystems;

pub(in crate::game) fn plugin(app: &mut App) {
    app.add_systems(
        Startup,
        (
            register_tile_assets.in_set(StartupSystems::RegisterManifests),
            populate_tile_assets.in_set(StartupSystems::PopulateAssets)
        )
    );
}

static GRASS_SPRITE: LazyLock<ResourceLocation<SpriteResource>> = LazyLock::new(|| "grass".parse().unwrap());
static PLANKS_SPRITE: LazyLock<ResourceLocation<SpriteResource>> = LazyLock::new(|| "planks".parse().unwrap());
static LIGHT_PLANKS_SPRITE: LazyLock<ResourceLocation<SpriteResource>> = LazyLock::new(|| "light_planks".parse().unwrap());
static FRAMED_PLANKS_SPRITE: LazyLock<ResourceLocation<SpriteResource>> = LazyLock::new(|| "framed_planks".parse().unwrap());
static LIGHT_FRAMED_PLANKS_SPRITE: LazyLock<ResourceLocation<SpriteResource>> = LazyLock::new(|| "light_framed_planks".parse().unwrap());

fn register_tile_assets(
    mut sprite_registry: ResMut<SpriteRegistry>
) {
    sprite_registry.insert_manifest(GRASS_SPRITE.clone());
    sprite_registry.insert_manifest(PLANKS_SPRITE.clone());
    sprite_registry.insert_manifest(LIGHT_PLANKS_SPRITE.clone());
    sprite_registry.insert_manifest(FRAMED_PLANKS_SPRITE.clone());
    sprite_registry.insert_manifest(LIGHT_FRAMED_PLANKS_SPRITE.clone());
}

static TILE_SPRITE_LAYOUT: LazyLock<TextureAtlasLayout> = LazyLock::new(|| TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 8, Some(UVec2::splat(1)), None));

fn populate_tile_assets(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let tile_assets = TileLayout {
        layout: asset_server.add(TILE_SPRITE_LAYOUT.clone()),
    };

    commands.insert_resource(tile_assets);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TileLayout {
    //#[dependency]
    layout: Handle<TextureAtlasLayout>,
}

impl TileLayout {
    pub fn layout(&self) -> Handle<TextureAtlasLayout> {
        self.layout.clone()
    }
}
