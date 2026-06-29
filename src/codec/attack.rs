use crate::data::prelude::*;
use bevy::prelude::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AttackCodec {
    pub format: u8,
    pub duration: u64,
    pub stamina_cost: usize,
    pub animation: ResourceLocation<AnimationResource>,
    pub particle_sprite: ResourceLocation<CharacterSpriteResource>,
}

impl AttackCodec {
    pub const LATEST_FORMAT: u8 = 1;
}

impl Default for AttackCodec {
    fn default() -> Self {
        AttackCodec {
            format: AttackCodec::LATEST_FORMAT,
            duration: 150,
            stamina_cost: 0,
            animation: "untitled".parse().unwrap(),
            particle_sprite: "untitled".parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AttackSetCodec {
    pub format: u8,
    pub attacks: Vec<ResourceLocation<AttackResource>>,
}