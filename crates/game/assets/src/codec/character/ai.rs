use bevy::prelude::*;
use maybe_fields::maybe_fields;
use serde::{Deserialize, Serialize};

#[maybe_fields]
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypePath)]
pub struct AiCodec {}