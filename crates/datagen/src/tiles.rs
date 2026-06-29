use game_core::datagen_api::*;
use crate::{create_dir, write_data, WriteError};

const LATEST_FORMAT: u8 = 1;

const GRASS_SPRITE_SHEET: &str = "grass";
const GRASS: &str = "grass";
const GRASS_LAYER: &str = "grass_layer";
const GRASS_STAIRS_TOP_LEFT: &str = "grass_stairs_top_left";
const GRASS_STAIRS_TOP_RIGHT: &str = "grass_stairs_top_right";
const GRASS_STAIRS_BOTTOM_LEFT: &str = "grass_stairs_bottom_left";
const GRASS_STAIRS_BOTTOM_RIGHT: &str = "grass_stairs_bottom_right";

const DIRT_SPRITE_SHEET: &str = "dirt";
const DIRT: &str = "dirt";
const DIRT_LAYER: &str = "dirt_layer";

const PLANKS_SPRITE_SHEET: &str = "planks";
const PLANKS: &str = "planks";
const PLANKS_LOWER: &str = "planks_lower";
const PLANKS_LAYER: &str = "planks_layer";
const PLANKS_LAYER_LOWER: &str = "planks_layer_lower";
const PLANKS_STAIRS_TOP_LEFT: &str = "planks_stairs_top_left";
const PLANKS_STAIRS_TOP_RIGHT: &str = "planks_stairs_top_right";
const PLANKS_STAIRS_BOTTOM_LEFT: &str = "planks_stairs_bottom_left";
const PLANKS_STAIRS_BOTTOM_RIGHT: &str = "planks_stairs_bottom_right";

const LIGHT_PLANKS_SPRITE_SHEET: &str = "light_planks";
const LIGHT_PLANKS: &str = "light_planks";
const LIGHT_PLANKS_LAYER: &str = "light_planks_layer";
const LIGHT_PLANKS_STAIRS_TOP_LEFT: &str = "light_planks_stairs_top_left";
const LIGHT_PLANKS_STAIRS_TOP_RIGHT: &str = "light_planks_stairs_top_right";
const LIGHT_PLANKS_STAIRS_BOTTOM_LEFT: &str = "light_planks_stairs_bottom_left";
const LIGHT_PLANKS_STAIRS_BOTTOM_RIGHT: &str = "light_planks_stairs_bottom_right";

pub fn generate_tiles() -> Result<(), WriteError> {
    create_dir(TileResource::ROOT_DIR)?;

    create_tile_data(TileData::new(GRASS, GRASS_SPRITE_SHEET, 0))?;
    create_tile_data(TileData::new(GRASS_LAYER, GRASS_SPRITE_SHEET, 2))?;
    create_tile_data(TileData::new(GRASS_STAIRS_TOP_LEFT, GRASS_SPRITE_SHEET, 24)
        .with_shape(TileShape::Stairs(TileFacing::NegX)))?;
    create_tile_data(TileData::new(GRASS_STAIRS_TOP_RIGHT, GRASS_SPRITE_SHEET, 25)
        .with_shape(TileShape::Stairs(TileFacing::NegZ)))?;
    create_tile_data(TileData::new(GRASS_STAIRS_BOTTOM_LEFT, GRASS_SPRITE_SHEET, 26)
        .with_shape(TileShape::Stairs(TileFacing::PosX)))?;
    create_tile_data(TileData::new(GRASS_STAIRS_BOTTOM_RIGHT, GRASS_SPRITE_SHEET, 27)
        .with_shape(TileShape::Stairs(TileFacing::PosZ)))?;

    create_tile_data(TileData::new(DIRT, DIRT_SPRITE_SHEET, 1))?;
    create_tile_data(TileData::new(DIRT_LAYER, DIRT_SPRITE_SHEET, 3))?;

    create_tile_data(TileData::new(PLANKS, PLANKS_SPRITE_SHEET, 0))?;
    create_tile_data(TileData::new(PLANKS_LAYER, PLANKS_SPRITE_SHEET, 2))?;
    create_tile_data(TileData::new(PLANKS_STAIRS_TOP_LEFT, PLANKS_SPRITE_SHEET, 24)
        .with_shape(TileShape::Stairs(TileFacing::NegX)))?;
    create_tile_data(TileData::new(PLANKS_STAIRS_TOP_RIGHT, PLANKS_SPRITE_SHEET, 25)
        .with_shape(TileShape::Stairs(TileFacing::NegZ)))?;
    create_tile_data(TileData::new(PLANKS_STAIRS_BOTTOM_LEFT, PLANKS_SPRITE_SHEET, 26)
        .with_shape(TileShape::Stairs(TileFacing::PosX)))?;
    create_tile_data(TileData::new(PLANKS_STAIRS_BOTTOM_RIGHT, PLANKS_SPRITE_SHEET, 27)
        .with_shape(TileShape::Stairs(TileFacing::PosZ)))?;

    create_tile_data(TileData::new(LIGHT_PLANKS, LIGHT_PLANKS_SPRITE_SHEET, 0))?;
    create_tile_data(TileData::new(LIGHT_PLANKS_LAYER, LIGHT_PLANKS_SPRITE_SHEET, 2))?;
    create_tile_data(TileData::new(LIGHT_PLANKS_STAIRS_TOP_LEFT, LIGHT_PLANKS_SPRITE_SHEET, 24)
        .with_shape(TileShape::Stairs(TileFacing::NegX)))?;
    create_tile_data(TileData::new(LIGHT_PLANKS_STAIRS_TOP_RIGHT, LIGHT_PLANKS_SPRITE_SHEET, 25)
        .with_shape(TileShape::Stairs(TileFacing::NegZ)))?;
    create_tile_data(TileData::new(LIGHT_PLANKS_STAIRS_BOTTOM_LEFT, LIGHT_PLANKS_SPRITE_SHEET, 26)
        .with_shape(TileShape::Stairs(TileFacing::PosX)))?;
    create_tile_data(TileData::new(LIGHT_PLANKS_STAIRS_BOTTOM_RIGHT, LIGHT_PLANKS_SPRITE_SHEET, 27)
        .with_shape(TileShape::Stairs(TileFacing::PosZ)))?;

    Ok(())
}

fn create_tile_data(
    tile_data: TileData,
) -> Result<(), WriteError> {
    let loc: ResourceLocation<TileResource> = tile_data.loc.parse().unwrap();
    let codec = TileCodec::from(tile_data);
    write_data(loc, &codec)
}

struct TileData {
    loc: &'static str,
    sprite_sheet: &'static str,
    index: u8,
    shape: Option<TileShape>,
}
impl TileData {
    pub fn new(loc: &'static str, sprite_sheet: &'static str, index: u8) -> Self {
        Self {
            loc,
            sprite_sheet,
            index,
            shape: None,
        }
    }

    pub fn with_shape(self, shape: TileShape) -> Self {
        Self {
            shape: Some(shape),
            ..self
        }
    }
}
impl From<TileData> for TileCodec {
    fn from(data: TileData) -> Self {
        TileCodec::new(LATEST_FORMAT, data.sprite_sheet.parse().unwrap(), data.index, data.shape.into())
    }
}