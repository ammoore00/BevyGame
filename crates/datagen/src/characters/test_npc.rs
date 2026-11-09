use crate::WriteError;
use crate::characters::{
    AnimationData, AttackData, AttackSetData, CharacterData, create_character,
};
use assets::codec::{ActionStateCodec, CapsuleCodec, ColliderCodec, ColliderDataCodec, DamageKind, FixedIntervalCodec, FrameDataCodec, HealthEventKind, HitboxCodec, KeyFrameCodec};
use std::collections::HashMap;
use crate::characters::attack::basic_attack;

pub(super) fn generate_test_npc() -> Result<(), WriteError> {
    let mut animation_map = HashMap::new();

    // Standard animations

    let idle = AnimationData::new(
        "test/idle",
        64,
        64,
        FrameDataCodec::FixedInterval(FixedIntervalCodec::new(12, 150)),
    )
    .with_image("player/idle");
    animation_map.insert(ActionStateCodec::Idle, idle);

    let walking = AnimationData::new(
        "test/walking",
        64,
        64,
        FrameDataCodec::FixedInterval(FixedIntervalCodec::new(8, 50)),
    )
    .with_image("player/walking");
    animation_map.insert(ActionStateCodec::Walking, walking);

    let running = AnimationData::new(
        "test/running",
        64,
        64,
        FrameDataCodec::FixedInterval(FixedIntervalCodec::new(8, 50)),
    )
    .with_image("player/running");
    animation_map.insert(ActionStateCodec::Running, running);

    let sprinting = AnimationData::new(
        "test/sprinting",
        64,
        64,
        FrameDataCodec::FixedInterval(FixedIntervalCodec::new(8, 35)),
    )
    .with_image("player/running");
    animation_map.insert(ActionStateCodec::Sprinting, sprinting);

    // Attacks
    
    let mut attacks = Vec::new();
    
    let basic_attack_loc = "test/basic_attack";
    let basic_attack_image = "player/basic_attack";
    
    let basic_attack = basic_attack(basic_attack_loc, basic_attack_image, true);
    attacks.push(basic_attack);
    
    let attack_set = AttackSetData::new(basic_attack_loc, attacks);

    let collider = ColliderDataCodec::Capsule(CapsuleCodec::Vertical {
        height: 1.25,
        radius: 0.25,
    });

    let data = CharacterData::new("test", collider)
        .with_animations(animation_map)
        .with_attack_set(attack_set);

    create_character(data)
}
