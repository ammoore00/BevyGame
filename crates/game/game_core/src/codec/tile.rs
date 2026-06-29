use crate::data::loader::Maybe;
use crate::game::level::grid::tile::TileShape;
use crate::prelude::TileSpriteResource;
use bevy::prelude::TypePath;
use game_data::prelude::*;
use maybe_fields::maybe_fields;
use serde::{Deserialize, Serialize};

#[maybe_fields]
#[derive(derive_new::new, Serialize, Deserialize, TypePath)]
pub struct TileCodec {
    pub format: u8,
    pub sprite_sheet: ResourceLocation<TileSpriteResource>,
    pub sprite_index: u8,
    pub shape: Maybe<TileShape>,
}