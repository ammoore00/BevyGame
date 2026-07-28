use crate::codec::{ColliderCodec, HealthEventKind};
use crate::loader::Maybe;
use crate::resource::characters::{AnimationResource, AttackResource, CharacterSpriteResource};
use bevy::prelude::*;
use data::prelude::*;
use maybe_fields::maybe_fields;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AttackCodec {
    pub format: u8,
    pub duration: u64,
    pub stamina_cost: usize,
    pub animation: ResourceLocation<AnimationResource>,
    pub particle_sprite: ResourceLocation<CharacterSpriteResource>,
    pub key_frames: Vec<KeyFrameCodec>,
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
            key_frames: Vec::new(),
        }
    }
}

#[maybe_fields]
#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct KeyFrameCodec {
    pub start_time: u64,
    pub end_time: u64,
    pub hitbox: HitboxCodec,

    pub health_event: HealthEventKind,
    pub disable_on_hit_iframes: Maybe<bool>,

    pub exclusion_group: Maybe<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub enum HitboxCodec {
    Static {
        collider: ColliderCodec,
        offset: Vec3,
    },
    Interpolated {
        collider_start: ColliderCodec,
        collider_end: ColliderCodec,
        offset_start: Vec3,
        offset_end: Vec3,
    },
    Swept {
        // TODO
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AttackSetCodec {
    pub format: u8,
    pub attacks: Vec<ResourceLocation<AttackResource>>,
}
