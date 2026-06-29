use std::collections::HashMap;
use bevy_game_2d::datagen_api::*;
use crate::characters::{create_character, AnimationData, AttackData, AttackSetData, CharacterData};
use crate::WriteError;

pub(super) fn generate_test_npc() -> Result<(), WriteError> {
    let mut animation_map = HashMap::new();

    // Standard animations

    let idle = AnimationData::new(
        "test/idle",
        64, 64,
        FrameDataCodec::FixedInterval {
            num_frames: 12,
            interval: 150,
        },
    ).with_image("player/idle");
    animation_map.insert(ActionStateEnum::Idle, idle);

    let walking = AnimationData::new(
        "test/walking",
        64, 64,
        FrameDataCodec::FixedInterval {
            num_frames: 8,
            interval: 50,
        },
    ).with_image("player/walking");
    animation_map.insert(ActionStateEnum::Walking, walking);

    let running = AnimationData::new(
        "test/running",
        64, 64,
        FrameDataCodec::FixedInterval {
            num_frames: 8,
            interval: 50,
        },
    ).with_image("player/running");
    animation_map.insert(ActionStateEnum::Running, running);

    let sprinting = AnimationData::new(
        "test/sprinting",
        64, 64,
        FrameDataCodec::FixedInterval {
            num_frames: 8,
            interval: 35,
        },
    ).with_image("player/running");
    animation_map.insert(ActionStateEnum::Sprinting, sprinting);

    // Attacks

    let mut attacks = Vec::new();

    let basic_attack_loc = "test/basic_attack";

    let basic_attack_length: u64 = 350;
    let basic_attack_frames: u32 = 7;
    let basic_attack_interval = basic_attack_length / basic_attack_frames as u64;

    let basic_attack_animation = AnimationData::new(
        "player/basic_attack",
        96, 96,
        FrameDataCodec::FixedInterval {
            num_frames: basic_attack_frames as usize,
            interval: basic_attack_interval,
        },
    );

    let basic_attack_stamina_cost = 20;

    let basic_attack = AttackData::new(
        basic_attack_loc,
        basic_attack_length,
        basic_attack_stamina_cost,
        basic_attack_animation,
        format!("{}_particle", basic_attack_loc).as_str()
    );
    attacks.push(basic_attack);

    let attack_set = AttackSetData::new(basic_attack_loc, attacks);

    let collider = ColliderKindCodec::Capsule(
        CapsuleCodec::Vertical {
            height: 1.25,
            radius: 0.25,
        }
    );

    let data = CharacterData::new("test", collider)
        .with_animations(animation_map)
        .with_attack_set(attack_set);

    create_character(data)
}