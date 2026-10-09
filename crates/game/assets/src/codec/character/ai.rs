use crate::loader::{Maybe, MaybeOrDefault};
use bevy::prelude::*;
use maybe_fields::maybe_fields;
use serde::{Deserialize, Serialize};

#[maybe_fields]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, TypePath)]
pub struct AiCodec {
    pub format: u8,
    pub senses: Maybe<AiSensesCodec>,
    pub leash_distance: Maybe<f32>,
}
impl AiCodec {
    pub const LATEST_FORMAT: u8 = 1;
}

#[maybe_fields]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, TypePath)]
pub struct AiSensesCodec {
    pub sight: MaybeOrDefault<AiSightCodec>,
    pub hearing: MaybeOrDefault<AiHearingCodec>,
}

#[maybe_fields]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, TypePath)]
pub struct AiSightCodec {
    pub range: Maybe<f32>,
    pub half_angle: Maybe<f32>,
}

#[maybe_fields]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, TypePath)]
pub struct AiHearingCodec {}
