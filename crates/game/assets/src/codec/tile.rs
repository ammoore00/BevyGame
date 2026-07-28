use crate::loader::Maybe;
use crate::resource::level::{TileShape, TileSpriteResource};
use bevy::prelude::TypePath;
use data::prelude::*;
use maybe_fields::maybe_fields;
use serde::{Deserialize, Serialize};

#[maybe_fields]
#[derive(Serialize, Deserialize, TypePath, derive_new::new)]
pub struct TileCodec {
    pub format: u8,
    pub sprite_sheet: ResourceLocation<TileSpriteResource>,
    pub sprite_index: u8,
    pub shape: Maybe<TileShape>,
}
