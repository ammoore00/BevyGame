use std::time::Duration;
use crate::data;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::data::loader::InlineOrResourceLocation;
use crate::data::ResourceFileType;
use crate::datagen_api::animation::{AnimationCodec, AnimationResource};
use crate::define_resource;

#[derive(Debug, Clone, Asset, TypePath)]
pub struct AttackDefinition {
    _length: Duration,
}
impl From<AttackCodec> for AttackDefinition {
    fn from(value: AttackCodec) -> Self {
        AttackDefinition {
            _length: Duration::from_millis(value.length),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AttackCodec {
    pub format: u8,
    pub length: u64,
    pub animation: InlineOrResourceLocation<AnimationResource, AnimationCodec>,
}

define_resource!(Attack, "characters/attacks", AttackDefinition, ResourceFileType::Data);