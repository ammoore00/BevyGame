mod animation;
mod attack;
mod character;
mod collider;
mod room;
mod sprite;
mod tile;

pub use crate::codec::{
    animation::{AnimationCodec, FrameDataCodec},
    attack::{AttackCodec, AttackSetCodec, KeyFrameCodec, HitboxCodec},
    character::{ActionStateCodec, AllowedStatesCodec, CharacterCodec},
    collider::{CapsuleCodec, ColliderCodec, ColliderDataCodec},
    room::RoomCodec,
    sprite::TextureAtlasCodec,
    tile::TileCodec,
};
