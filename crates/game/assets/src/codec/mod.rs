pub mod animation;
mod character;
mod collider;
mod room;
mod sprite;
mod tile;
mod translation;

pub use crate::codec::{
    animation::{AnimationCodec, DistinctCodec, FixedIntervalCodec, FrameDataCodec},
    character::{
        ActionStateCodec, AllowedStatesCodec, CharacterCodec,
        ai::{AiCodec, AiHearingCodec, AiSensesCodec, AiSightCodec},
        attack::{AttackCodec, AttackSetCodec, HitboxCodec, KeyFrameCodec},
        health::{
            DamageKind, DamageModifierCodec, DamageModifierKind, DamageModifierList, HealthCodec,
            HealthEventKind, ModifierTier,
        },
    },
    collider::{CapsuleCodec, ColliderCodec, ColliderDataCodec},
    room::RoomCodec,
    sprite::TextureAtlasCodec,
    tile::TileCodec,
    translation::{TranslationCodec, TranslationList},
};
