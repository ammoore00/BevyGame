use std::io::Write;
use bevy_game_2d::data::ResourceLocation;
use bevy_game_2d::datagen_api::tile::codec::{TileCodec, TileResource};
use crate::{ROOT, ROOT_BASE};

pub fn generate_tiles() -> Result<(), TileError> {
    std::fs::create_dir_all(ROOT_BASE.join("tiles")).expect("Failed to create tile directory");

    create_tile_data(DIRT)?;
    create_tile_data(DIRT_LAYER)?;

    create_tile_data(GRASS)?;
    create_tile_data(GRASS_LAYER)?;

    Ok(())
}

fn create_tile_data(
    tile_data: TileData,
) -> Result<(), TileError> {
    let loc: ResourceLocation<TileResource> = tile_data.loc.parse().unwrap();
    let codec = TileCodec::from(tile_data);

    let file = std::fs::File::create(ROOT.join(loc.as_path()))?;
    let mut writer = std::io::BufWriter::new(file);

    let serialized = ron::ser::to_string_pretty(&codec, ron::ser::PrettyConfig::default())?;
    writer.write_all(serialized.as_bytes())?;

    Ok(())
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

#[derive(Debug, thiserror::Error)]
pub enum TileError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RON serialize error: {0}")]
    Ron(#[from] ron::Error),
}

const LATEST_FORMAT: u8 = 1;

const DIRT: TileData = TileData::new("dirt", "grass", 1);
const DIRT_LAYER: TileData = TileData::new("dirt_layer", "grass", 3);

const GRASS: TileData = TileData::new("grass", "grass", 0);
const GRASS_LAYER: TileData = TileData::new("grass_layer", "grass", 2);