use crate::resource::TextResource;
use bevy::prelude::*;
use data::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct TranslationCodec {
    pub format: u8,
    pub ui: TranslationList<TextResource>,
}
impl TranslationCodec {
    const LATEST_FORMAT: u8 = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    transparent,
    bound(
        serialize = "ResourceLocation<T>: Serialize",
        deserialize = "ResourceLocation<T>: Deserialize<'de>"
    )
)]
pub struct TranslationList<T: ResourceKind>(pub HashMap<ResourceLocation<T>, String>);
