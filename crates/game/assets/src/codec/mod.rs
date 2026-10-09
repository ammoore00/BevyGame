mod character;
mod collider;
mod room;
mod sprite;
mod tile;
mod translation;

pub use crate::codec::{
    character::{
        ActionStateCodec, AllowedStatesCodec, CharacterCodec,
        ai::{AiCodec, AiHearingCodec, AiSensesCodec, AiSightCodec},
        animation::{AnimationCodec, FrameDataCodec},
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
