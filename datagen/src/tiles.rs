use bevy_game_2d::data::{ResourceLocation, ResourceType};
use bevy_game_2d::datagen_api::tile::{TileCodec, TileResource};
use crate::{create_dir, write_data, WriteError};

pub fn generate_tiles() -> Result<(), WriteError> {
    create_dir(TileResource::root_dir())?;

    create_tile_data(DIRT)?;
    create_tile_data(DIRT_LAYER)?;

    create_tile_data(GRASS)?;
    create_tile_data(GRASS_LAYER)?;

    create_tile_data(PLANKS)?;
    create_tile_data(PLANKS_LAYER)?;

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
    index: u8
}
impl TileData {
    const fn new(loc: &'static str, sprite_sheet: &'static str, index: u8) -> Self {
        Self { loc, sprite_sheet, index }
    }
}
impl From<TileData> for TileCodec {
    fn from(data: TileData) -> Self {
        TileCodec::new(LATEST_FORMAT, data.sprite_sheet.parse().unwrap(), data.index)
    }
}

const LATEST_FORMAT: u8 = 1;

const DIRT: TileData = TileData::new("dirt", "grass", 1);
const DIRT_LAYER: TileData = TileData::new("dirt_layer", "grass", 3);

const GRASS: TileData = TileData::new("grass", "grass", 0);
const GRASS_LAYER: TileData = TileData::new("grass_layer", "grass", 2);

const PLANKS: TileData = TileData::new("planks", "planks", 0);
const PLANKS_LAYER: TileData = TileData::new("planks_layer", "planks", 2);