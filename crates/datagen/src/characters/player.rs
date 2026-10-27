use crate::WriteError;
use crate::characters::{
    AnimationData, AttackData, AttackSetData, CharacterData, create_character,
};
use assets::codec::{
    ActionStateCodec, CapsuleCodec, ColliderCodec, ColliderDataCodec, DamageKind, FrameDataCodec,
    HealthEventKind, HitboxCodec, KeyFrameCodec,
};
use std::collections::HashMap;
use crate::characters::attack::basic_attack;

pub(super) fn generate_player() -> Result<(), WriteError> {
    let mut animation_map = HashMap::new();

    // Standard animations

    let idle = AnimationData::new(
        "player/idle",
        64,
        64,
        FrameDataCodec::FixedInterval {
            num_frames: 12,
            interval: 150,
        },
    );
    animation_map.insert(ActionStateCodec::Idle, idle);

    let walking = AnimationData::new(
        "player/walking",
        64,
        64,
        FrameDataCodec::FixedInterval {
            num_frames: 8,
            interval: 50,
        },
    );
    animation_map.insert(ActionStateCodec::Walking, walking);

    let running = AnimationData::new(
        "player/running",
        64,
        64,
        FrameDataCodec::FixedInterval {
            num_frames: 8,
            interval: 50,
        },
    );
    animation_map.insert(ActionStateCodec::Running, running);

    let sprinting = AnimationData::new(
        "player/sprinting",
        64,
        64,
        FrameDataCodec::FixedInterval {
            num_frames: 8,
            interval: 35,
        },
    )
    .with_image("player/running");
    animation_map.insert(ActionStateCodec::Sprinting, sprinting);

    let dodging = AnimationData::new(
        "player/rolling",
        48,
        48,
        FrameDataCodec::FixedInterval {
            num_frames: 11,
            interval: 35,
        },
    )
        .with_image("player/rolling");
    animation_map.insert(ActionStateCodec::Dodging, dodging);

    // Attacks

    let mut attacks = Vec::new();

    let basic_attack_loc = "player/basic_attack";
    let basic_attack_image = "player/basic_attack";

    let basic_attack = basic_attack(basic_attack_loc, basic_attack_image);
    attacks.push(basic_attack);

    let attack_set = AttackSetData::new(basic_attack_loc, attacks);

    let collider = ColliderDataCodec::Capsule(CapsuleCodec::Vertical {
        height: 1.25,
        radius: 0.25,
    });

    let data = CharacterData::new("player", collider)
        .with_animations(animation_map)
        .with_attack_set(attack_set)
        .with_ai_params(None);

    create_character(data)
}
