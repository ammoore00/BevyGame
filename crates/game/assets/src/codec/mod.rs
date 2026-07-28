mod animation;
mod attack;
mod character;
mod collider;
mod room;
mod sprite;
mod tile;
mod health;

pub use crate::codec::{
    animation::{AnimationCodec, FrameDataCodec},
    attack::{AttackCodec, AttackSetCodec, KeyFrameCodec, HitboxCodec},
    character::{ActionStateCodec, AllowedStatesCodec, CharacterCodec},
    collider::{CapsuleCodec, ColliderCodec, ColliderDataCodec},
    health::{DamageModifierCodec, DamageModifierKind, ModifierTier, DamageKind, HealthEventKind},
    room::RoomCodec,
    sprite::TextureAtlasCodec,
    tile::TileCodec,
};
