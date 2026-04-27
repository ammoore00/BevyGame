use std::sync::LazyLock;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::data::registry::ResourceRegistry;
use crate::data::{ResourceFileType, ResourceLocation, ResourceType};
use crate::data::loader::{LoaderJobManager, Maybe};
use crate::datagen_api::tile::TileShape;
use crate::StartupSystems;

pub(in crate::game) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<TileSpriteResource>();

    app.add_systems(
        Startup,
        populate_tile_assets.in_set(StartupSystems::PopulateAssets)
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

pub type TileRegistry = ResourceRegistry<TileResource>;

#[derive(derive_new::new, Serialize, Deserialize, TypePath)]
pub struct TileCodec {
    pub format: u8,
    pub sprite_sheet: ResourceLocation<TileSpriteResource>,
    pub sprite_index: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Maybe<TileShape>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Default, Reflect)]
pub struct TileResource;

impl ResourceType for TileResource {
    type AssetType = TileAsset;
    const ROOT_DIR: &'static str = "tiles";
    const FILE_TYPE: ResourceFileType = ResourceFileType::Data;
}

#[derive(Debug, Clone, Asset, TypePath)]
pub struct TileAsset {
    sprite_sheet: ResourceLocation<TileSpriteResource>,
    sprite_index: u8,
    pub shape: TileShape,
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

pub type TileSpriteRegistry = ResourceRegistry<TileSpriteResource>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct TileSpriteResource;
impl ResourceType for TileSpriteResource {
    type AssetType = Image;
    const ROOT_DIR: &'static str = "images/tiles";
    const FILE_TYPE: ResourceFileType = ResourceFileType::Image;
}