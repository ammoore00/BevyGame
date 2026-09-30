use crate::resource::UiTextResource;
use bevy::prelude::*;
use data::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct TranslationCodec {
    pub format: u8,
    pub ui: TranslationList<UiTextResource>,
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
impl <T: ResourceKind> TranslationList<T> {
    pub fn translate(&self, loc: &ResourceLocation<T>) -> Option<&str> {
        self.0.get(loc).map(|s| s.as_str())
    }
}