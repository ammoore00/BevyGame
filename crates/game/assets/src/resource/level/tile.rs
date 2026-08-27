use crate::codec::TileCodec;
use crate::loader::{LoaderJobManager, RonAssetLoader};
use crate::state::AssetSystems;
use bevy::prelude::*;
use common::TILE_WIDTH;
use data::prelude::*;
use data::{define_data_resource, define_sprite_resource};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub(in crate::resource) fn plugin(app: &mut App) {
    app.init_asset::<TileAsset>();
    app.init_asset_loader::<RonAssetLoader<TileCodec, TileAsset>>();
    app.add_registry_with_discovery::<TileResource>();
    app.add_registry_with_discovery::<TileSpriteResource>();

    app.add_systems(
        Startup,
        populate_tile_assets.in_set(AssetSystems::PopulateAssetRefs),
    );
}

const TILE_SHEET_COLUMNS: u32 = 16;
const TILE_SHEET_ROWS: u32 = 16;

static TILE_SPRITE_LAYOUT: LazyLock<TextureAtlasLayout> = LazyLock::new(|| {
    TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_WIDTH as u32),
        TILE_SHEET_COLUMNS,
        TILE_SHEET_ROWS,
        // Padding
        Some(UVec2::splat(1)),
        // Offset
        None,
    )
});

fn populate_tile_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
    let tile_assets = TileLayout {
        layout: asset_server.add(TILE_SPRITE_LAYOUT.clone()),
    };

    commands.insert_resource(tile_assets);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TileLayout {
    layout: Handle<TextureAtlasLayout>,
}
impl TileLayout {
    pub fn layout(&self) -> Handle<TextureAtlasLayout> {
        self.layout.clone()
    }
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

define_data_resource!(Tile, "tiles", TileAsset);
define_sprite_resource!(Tile, "tiles");

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TileShape {
    #[default]
    Full,
    SlopeLower(TileFacing),
    SlopeUpper(TileFacing),
    Stairs(TileFacing),
    Bridge(Option<TileFacing>),
    Other(Vec<Vec3>),
}

impl PartialEq for TileShape {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TileShape::Full, TileShape::Full) => true,
            (TileShape::SlopeLower(a), TileShape::SlopeLower(b)) => a == b,
            (TileShape::SlopeUpper(a), TileShape::SlopeUpper(b)) => a == b,
            (TileShape::Stairs(a), TileShape::Stairs(b)) => a == b,
            (TileShape::Bridge(a), TileShape::Bridge(b)) => a == b,
            (TileShape::Other(a), TileShape::Other(b)) => {
                let sorted = |v: &[Vec3]| -> Vec<[f32; 3]> {
                    let mut s: Vec<[f32; 3]> = v.iter().map(|v| [v.x, v.y, v.z]).collect();

                    s.sort_by(|a, b| {
                        a[0].total_cmp(&b[0])
                            .then(a[1].total_cmp(&b[1]))
                            .then(a[2].total_cmp(&b[2]))
                    });

                    s
                };

                let mut a = a.clone();
                a.dedup();

                let mut b = b.clone();
                b.dedup();

                a.len() == b.len() && sorted(&a) == sorted(&b)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileFacing {
    PosX,
    NegX,
    PosZ,
    NegZ,
}

impl TileFacing {
    /// Returns the rotation angle in radians around the Y axis for this facing direction
    pub fn rotation_y(&self) -> f32 {
        match self {
            TileFacing::PosX => 0.0,
            TileFacing::PosZ => std::f32::consts::FRAC_PI_2, // 90 degrees
            TileFacing::NegX => std::f32::consts::PI,        // 180 degrees
            TileFacing::NegZ => -std::f32::consts::FRAC_PI_2, // -90 degrees (or 270)
        }
    }

    /// Rotates a point around the Y axis according to this facing direction
    pub fn rotate_point(&self, point: Vec3) -> Vec3 {
        let angle = self.rotation_y();
        let cos = angle.cos();
        let sin = angle.sin();

        Vec3::new(
            point.x * cos - point.z * sin,
            point.y,
            point.x * sin + point.z * cos,
        )
    }
}
