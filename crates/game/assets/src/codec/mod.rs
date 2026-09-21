mod character;
mod collider;
mod room;
mod sprite;
mod tile;

pub use crate::codec::{
    character::{
        ActionStateCodec, AllowedStatesCodec, CharacterCodec,
        animation::{AnimationCodec, FrameDataCodec},
        attack::{AttackCodec, AttackSetCodec, HitboxCodec, KeyFrameCodec},
        health::{
            DamageKind, DamageModifierCodec, DamageModifierKind, DamageModifierList,
            HealthEventKind, ModifierTier,
        },
    },
    collider::{CapsuleCodec, ColliderCodec, ColliderDataCodec},
    room::RoomCodec,
    sprite::TextureAtlasCodec,
    tile::TileCodec,
};
