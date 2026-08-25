use crate::{WriteError, create_dir, write_data};
use assets::codec::TileCodec;
use assets::resource::level::{TileFacing, TileResource, TileShape};
use data::prelude::*;

const LATEST_FORMAT: u8 = 1;

const GRASS_SPRITE_SHEET: &str = "grass";

pub const GRASS_ALL_OUTLINE: &str = "grass_all_outline";
pub const GRASS_ALL: &str = "grass_all";
pub const GRASS_S_OUTLINE: &str = "grass_s_outline";
pub const GRASS_S: &str = "grass_s";
pub const GRASS_SW_OUTLINE: &str = "grass_sw_outline";
pub const GRASS_SW: &str = "grass_sw";
pub const GRASS_W_OUTLINE: &str = "grass_w_outline";
pub const GRASS_W: &str = "grass_w";
pub const GRASS_NW_OUTLINE: &str = "grass_nw_outline";
pub const GRASS_NW: &str = "grass_nw";
pub const GRASS_N: &str = "grass_n";
pub const GRASS_NE: &str = "grass_ne";
pub const GRASS_E: &str = "grass_e";
pub const GRASS_SE_OUTLINE: &str = "grass_se_outline";
pub const GRASS_SE: &str = "grass_se";
pub const GRASS_NS_OUTLINE: &str = "grass_ns_outline";
pub const GRASS_NS: &str = "grass_ns";
pub const GRASS_EW_OUTLINE: &str = "grass_ew_outline";
pub const GRASS_EW: &str = "grass_ew";
pub const GRASS_NOT_S_OUTLINE: &str = "grass_not_s_outline";
pub const GRASS_NOT_S: &str = "grass_not_s";
pub const GRASS_NOT_W_OUTLINE: &str = "grass_not_w_outline";
pub const GRASS_NOT_W: &str = "grass_not_w";
pub const GRASS_NOT_N_OUTLINE: &str = "grass_not_n_outline";
pub const GRASS_NOT_N: &str = "grass_not_n";
pub const GRASS_NOT_E_OUTLINE: &str = "grass_not_e_outline";
pub const GRASS_NOT_E: &str = "grass_not_e";
pub const GRASS: &str = "grass";

pub fn generate_tiles() -> Result<(), WriteError> {
    create_dir(TileResource::ROOT_DIR)?;

    create_tile_data(TileData::new(GRASS_ALL_OUTLINE, GRASS_SPRITE_SHEET, 0))?;
    create_tile_data(TileData::new(GRASS_ALL, GRASS_SPRITE_SHEET, 1))?;
    create_tile_data(TileData::new(GRASS_S_OUTLINE, GRASS_SPRITE_SHEET, 2))?;
    create_tile_data(TileData::new(GRASS_S, GRASS_SPRITE_SHEET, 3))?;
    create_tile_data(TileData::new(GRASS_SW_OUTLINE, GRASS_SPRITE_SHEET, 4))?;
    create_tile_data(TileData::new(GRASS_SW, GRASS_SPRITE_SHEET, 5))?;
    create_tile_data(TileData::new(GRASS_W_OUTLINE, GRASS_SPRITE_SHEET, 6))?;
    create_tile_data(TileData::new(GRASS_W, GRASS_SPRITE_SHEET, 7))?;
    create_tile_data(TileData::new(GRASS_NW_OUTLINE, GRASS_SPRITE_SHEET, 8))?;
    create_tile_data(TileData::new(GRASS_NW, GRASS_SPRITE_SHEET, 9))?;
    create_tile_data(TileData::new(GRASS_N, GRASS_SPRITE_SHEET, 10))?;
    create_tile_data(TileData::new(GRASS_NE, GRASS_SPRITE_SHEET, 11))?;
    create_tile_data(TileData::new(GRASS_E, GRASS_SPRITE_SHEET, 12))?;
    create_tile_data(TileData::new(GRASS_SE_OUTLINE, GRASS_SPRITE_SHEET, 13))?;
    create_tile_data(TileData::new(GRASS_SE, GRASS_SPRITE_SHEET, 14))?;
    create_tile_data(TileData::new(GRASS_NS_OUTLINE, GRASS_SPRITE_SHEET, 15))?;
    create_tile_data(TileData::new(GRASS_NS, GRASS_SPRITE_SHEET, 16))?;
    create_tile_data(TileData::new(GRASS_EW_OUTLINE, GRASS_SPRITE_SHEET, 17))?;
    create_tile_data(TileData::new(GRASS_EW, GRASS_SPRITE_SHEET, 18))?;
    create_tile_data(TileData::new(GRASS_NOT_S_OUTLINE, GRASS_SPRITE_SHEET, 19))?;
    create_tile_data(TileData::new(GRASS_NOT_S, GRASS_SPRITE_SHEET, 20))?;
    create_tile_data(TileData::new(GRASS_NOT_W_OUTLINE, GRASS_SPRITE_SHEET, 21))?;
    create_tile_data(TileData::new(GRASS_NOT_W, GRASS_SPRITE_SHEET, 22))?;
    create_tile_data(TileData::new(GRASS_NOT_N_OUTLINE, GRASS_SPRITE_SHEET, 23))?;
    create_tile_data(TileData::new(GRASS_NOT_N, GRASS_SPRITE_SHEET, 24))?;
    create_tile_data(TileData::new(GRASS_NOT_E_OUTLINE, GRASS_SPRITE_SHEET, 25))?;
    create_tile_data(TileData::new(GRASS_NOT_E, GRASS_SPRITE_SHEET, 26))?;
    create_tile_data(TileData::new(GRASS, GRASS_SPRITE_SHEET, 27))?;

    Ok(())
}

fn create_tile_data(tile_data: TileData) -> Result<(), WriteError> {
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
        TileCodec::new(
            LATEST_FORMAT,
            data.sprite_sheet.parse().unwrap(),
            data.index,
            data.shape.into(),
        )
    }
}
