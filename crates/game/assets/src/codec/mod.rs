mod animation;
mod attack;
mod character;
mod collider;
mod health;
mod room;
mod sprite;
mod tile;

pub use crate::codec::{
    animation::{AnimationCodec, FrameDataCodec},
    attack::{AttackCodec, AttackSetCodec, HitboxCodec, KeyFrameCodec},
    character::{ActionStateCodec, AllowedStatesCodec, CharacterCodec},
    collider::{CapsuleCodec, ColliderCodec, ColliderDataCodec},
    health::{DamageKind, DamageModifierCodec, DamageModifierKind, HealthEventKind, ModifierTier},
    room::RoomCodec,
    sprite::TextureAtlasCodec,
    tile::TileCodec,
};
