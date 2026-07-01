use crate::game::level::grid::tile::assets::TileSpriteResource;
use crate::game::level::grid::tile::TileShape;
use assets::Maybe;
use bevy::prelude::TypePath;
use data::prelude::*;
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