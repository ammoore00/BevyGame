use crate::game::grid::tile::TileShape;
use crate::game::grid::tile::assets::TileMaterial;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileType {
    pub shape: TileShape,
    pub material: TileMaterial,
    pub index: usize,
}

static ALL_TILES: LazyLock<Vec<TileType>> = LazyLock::new(|| {
    let mut vec = Vec::new();
    vec.extend_from_slice(grass::STANDARD_SET);
    vec.extend_from_slice(dark_planks::STANDARD_SET);
    vec.extend_from_slice(dark_framed_planks::STANDARD_SET);
    vec.extend_from_slice(light_planks::STANDARD_SET);
    vec.extend_from_slice(light_framed_planks::STANDARD_SET);
    vec
});

impl TileType {
    const fn new(shape: TileShape, material: TileMaterial, index: usize) -> Self {
        Self {
            shape,
            material,
            index,
        }
    }

    pub fn get_tile(shape: TileShape, material: TileMaterial) -> Self {
        *ALL_TILES
            .iter()
            .find(|tile| tile.shape == shape && tile.material == material)
            .unwrap_or_else(|| {
                panic!("No tile with the given shape: {shape:?} and material: {material:?} exists")
            })
    }
}

const FULL_INDICES: &[usize] = &[0, 1];
const LAYER_INDICES: &[usize] = &[2, 3];

const SLOPE_LOWER_INDICES: &[usize] = &[8, 9, 10, 11];
const SLOPE_UPPER_INDICES: &[usize] = &[12, 13, 14, 15];

const STAIRS_INDICES: &[usize] = &[24, 25, 26, 27];

const BRIDGE_INDICES: &[usize] = &[32, 33, 34, 35, 36];

macro_rules! standard_tile_set {
    ($material:ident) => {
        pub const FULL: TileType =
            TileType::new(TileShape::Full { is_top: true }, $material, FULL_INDICES[0]);
        pub const FULL_BOTTOM: TileType = TileType::new(
            TileShape::Full { is_top: false },
            $material,
            FULL_INDICES[1],
        );

        pub const LAYER: TileType = TileType::new(
            TileShape::Layer { is_top: true },
            $material,
            LAYER_INDICES[0],
        );
        pub const LAYER_BOTTOM: TileType = TileType::new(
            TileShape::Layer { is_top: false },
            $material,
            LAYER_INDICES[1],
        );

        pub const SLOPE_LOWER_NEG_X: TileType = TileType::new(
            TileShape::SlopeLower(TileFacing::NegX),
            $material,
            SLOPE_LOWER_INDICES[0],
        );
        pub const SLOPE_LOWER_NEG_Z: TileType = TileType::new(
            TileShape::SlopeLower(TileFacing::NegZ),
            $material,
            SLOPE_LOWER_INDICES[1],
        );
        pub const SLOPE_LOWER_POS_X: TileType = TileType::new(
            TileShape::SlopeLower(TileFacing::PosX),
            $material,
            SLOPE_LOWER_INDICES[2],
        );
        pub const SLOPE_LOWER_POS_Z: TileType = TileType::new(
            TileShape::SlopeLower(TileFacing::PosZ),
            $material,
            SLOPE_LOWER_INDICES[3],
        );

        pub const SLOPE_UPPER_NEG_X: TileType = TileType::new(
            TileShape::SlopeUpper(TileFacing::NegX),
            $material,
            SLOPE_UPPER_INDICES[0],
        );
        pub const SLOPE_UPPER_NEG_Z: TileType = TileType::new(
            TileShape::SlopeUpper(TileFacing::NegZ),
            $material,
            SLOPE_UPPER_INDICES[1],
        );
        pub const SLOPE_UPPER_POS_X: TileType = TileType::new(
            TileShape::SlopeUpper(TileFacing::PosX),
            $material,
            SLOPE_UPPER_INDICES[2],
        );
        pub const SLOPE_UPPER_POS_Z: TileType = TileType::new(
            TileShape::SlopeUpper(TileFacing::PosZ),
            $material,
            SLOPE_UPPER_INDICES[3],
        );

        pub const STAIRS_NEG_X: TileType = TileType::new(
            TileShape::Stairs(TileFacing::NegX),
            $material,
            STAIRS_INDICES[0],
        );
        pub const STAIRS_NEG_Z: TileType = TileType::new(
            TileShape::Stairs(TileFacing::NegZ),
            $material,
            STAIRS_INDICES[1],
        );
        pub const STAIRS_POS_X: TileType = TileType::new(
            TileShape::Stairs(TileFacing::PosX),
            $material,
            STAIRS_INDICES[2],
        );
        pub const STAIRS_POS_Z: TileType = TileType::new(
            TileShape::Stairs(TileFacing::PosZ),
            $material,
            STAIRS_INDICES[3],
        );

        pub const BRIDGE: TileType =
            TileType::new(TileShape::Bridge(None), $material, BRIDGE_INDICES[0]);
        pub const BRIDGE_NEG_X: TileType = TileType::new(
            TileShape::Bridge(Some(TileFacing::NegX)),
            $material,
            BRIDGE_INDICES[1],
        );
        pub const BRIDGE_NEG_Z: TileType = TileType::new(
            TileShape::Bridge(Some(TileFacing::NegZ)),
            $material,
            BRIDGE_INDICES[2],
        );
        pub const BRIDGE_POS_X: TileType = TileType::new(
            TileShape::Bridge(Some(TileFacing::PosX)),
            $material,
            BRIDGE_INDICES[3],
        );
        pub const BRIDGE_POS_Z: TileType = TileType::new(
            TileShape::Bridge(Some(TileFacing::PosZ)),
            $material,
            BRIDGE_INDICES[4],
        );

        pub const STANDARD_SET: &[TileType] = &[
            FULL,
            FULL_BOTTOM,
            LAYER,
            LAYER_BOTTOM,
            SLOPE_LOWER_POS_X,
            SLOPE_LOWER_POS_Z,
            SLOPE_LOWER_NEG_X,
            SLOPE_LOWER_NEG_Z,
            SLOPE_UPPER_POS_X,
            SLOPE_UPPER_POS_Z,
            SLOPE_UPPER_NEG_X,
            SLOPE_UPPER_NEG_Z,
            STAIRS_POS_X,
            STAIRS_POS_Z,
            STAIRS_NEG_X,
            STAIRS_NEG_Z,
            BRIDGE,
            BRIDGE_POS_X,
            BRIDGE_POS_Z,
            BRIDGE_NEG_X,
            BRIDGE_NEG_Z,
        ];
    };
}

pub mod grass {
    use super::*;
    use crate::game::grid::tile::TileFacing;

    const MATERIAL: TileMaterial = TileMaterial::Grass;
    standard_tile_set!(MATERIAL);
}

pub mod dark_planks {
    use super::*;
    use crate::game::grid::tile::TileFacing;

    const MATERIAL: TileMaterial = TileMaterial::DarkPlanks;
    standard_tile_set!(MATERIAL);
}

pub mod dark_framed_planks {
    use super::*;
    use crate::game::grid::tile::TileFacing;

    const MATERIAL: TileMaterial = TileMaterial::DarkFramedPlanks;
    standard_tile_set!(MATERIAL);
}

pub mod light_planks {
    use super::*;
    use crate::game::grid::tile::TileFacing;

    const MATERIAL: TileMaterial = TileMaterial::LightPlanks;
    standard_tile_set!(MATERIAL);
}

pub mod light_framed_planks {
    use super::*;
    use crate::game::grid::tile::TileFacing;

    const MATERIAL: TileMaterial = TileMaterial::LightFramedPlanks;
    standard_tile_set!(MATERIAL);
}
