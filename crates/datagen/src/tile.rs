use crate::{WriteError, write_data};
use assets::codec::TileCodec;
use assets::resource::level::{TileResource, TileShape};
use data::prelude::*;

const LATEST_FORMAT: u8 = 1;

pub fn create_tile_data(tile_data: TileData) -> Result<(), WriteError> {
    let loc: ResourceLocation<TileResource> = tile_data.loc.parse().unwrap();
    let codec = TileCodec::from(tile_data);
    write_data(loc, &codec)
}

pub struct TileData {
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
