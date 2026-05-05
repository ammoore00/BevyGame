use crate::data::loader::{LoaderJobManager, Maybe, RonAssetLoader};
use crate::data::{ResourceFileType, ResourceLocation};
use crate::datagen_api::tile::TileShape;
use crate::{data, define_data_resource};
use crate::{define_resource, define_sprite_resource, AssetSystems};
use bevy::prelude::*;
use maybe_fields_macro::maybe_fields;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub(in crate::game) fn plugin(app: &mut App) {
    app.init_asset::<TileAsset>();
    app.init_asset_loader::<RonAssetLoader<TileCodec, TileAsset>>();
    app.add_registry_with_discovery::<TileResource>();
    app.add_registry_with_discovery::<TileSpriteResource>();

    app.add_systems(
        Startup,
        populate_tile_assets.in_set(AssetSystems::PopulateAssetRefs)
    );
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

#[maybe_fields]
#[derive(derive_new::new, Serialize, Deserialize, TypePath)]
pub struct TileCodec {
    pub format: u8,
    pub sprite_sheet: ResourceLocation<TileSpriteResource>,
    pub sprite_index: u8,
    pub shape: Maybe<TileShape>,
}

#[derive(Debug, Clone, Asset, TypePath)]
pub struct TileAsset {
    sprite_sheet: ResourceLocation<TileSpriteResource>,
    sprite_index: u8,
    pub shape: TileShape,
}
impl TileAsset {
    pub fn sprite_sheet(&self) -> &ResourceLocation<TileSpriteResource> {
        &self.sprite_sheet
    }
    pub fn sprite_index(&self) -> u8 {
        self.sprite_index
    }
    pub fn shape(&self) -> &TileShape {
        &self.shape
    }
}
impl From<TileCodec> for TileAsset {
    fn from(codec: TileCodec) -> Self {
        Self {
            sprite_sheet: codec.sprite_sheet,
            sprite_index: codec.sprite_index,
            shape: codec.shape.into_inner().unwrap_or_default(),
        }
    }
}


define_data_resource!(Tile, "tiles", TileAsset, ResourceFileType::Data);
define_sprite_resource!(Tile, "tiles");