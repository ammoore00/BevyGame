use crate::data::loader::Maybe;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! define_sprite_resource {
    ($name:ident, $path:literal) => {
        paste::paste! {
            $crate::define_resource!(
                [<$name Sprite>],
                concat!("images/", $path),
                Image,
                ResourceFileType::Image
            );
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct TextureAtlasCodec {
    pub format: u8,
    pub size: UVec2,
    pub columns: u32,
    pub rows: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Maybe<UVec2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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