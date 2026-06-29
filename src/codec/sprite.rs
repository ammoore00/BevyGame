use crate::data::loader::Maybe;
use bevy::image::TextureAtlasLayout;
use bevy::math::UVec2;
use bevy::prelude::TypePath;
use serde::{Deserialize, Serialize};
use maybe_fields_macro::maybe_fields;

#[maybe_fields]
#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct TextureAtlasCodec {
    pub format: u8,
    pub size: UVec2,
    pub columns: u32,
    pub rows: u32,
    pub padding: Maybe<UVec2>,
    pub offset: Maybe<UVec2>,
}

impl TextureAtlasCodec {
    pub const LATEST_FORMAT: u8 = 1;
}

impl From<TextureAtlasCodec> for TextureAtlasLayout {
    fn from(codec: TextureAtlasCodec) -> Self {
        TextureAtlasLayout::from_grid(codec.size, codec.columns, codec.rows, codec.padding.into(), codec.offset.into())
    }
}