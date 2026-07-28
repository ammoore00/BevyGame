use std::collections::HashMap;
use assets::codec::{ActionStateCodec, CapsuleCodec, ColliderCodec, ColliderDataCodec, DamageKind, FrameDataCodec, HealthEventKind, HitboxCodec, KeyFrameCodec};
use crate::characters::{create_character, AnimationData, AttackData, AttackSetData, CharacterData};
use crate::WriteError;

pub(super) fn generate_player() -> Result<(), WriteError> {
    let mut animation_map = HashMap::new();

    // Standard animations

    let idle = AnimationData::new(
        "player/idle",
        64, 64,
        FrameDataCodec::FixedInterval {
            num_frames: 12,
            interval: 150,
        },
    );
    animation_map.insert(ActionStateCodec::Idle, idle);

    let walking = AnimationData::new(
        "player/walking",
        64, 64,
        FrameDataCodec::FixedInterval {
            num_frames: 8,
            interval: 50,
        },
    );
    animation_map.insert(ActionStateCodec::Walking, walking);

    let running = AnimationData::new(
        "player/running",
        64, 64,
        FrameDataCodec::FixedInterval {
            num_frames: 8,
            interval: 50,
        },
    );
    animation_map.insert(ActionStateCodec::Running, running);

    let sprinting = AnimationData::new(
        "player/sprinting",
        64, 64,
        FrameDataCodec::FixedInterval {
            num_frames: 8,
            interval: 35,
        },
    ).with_image("player/running");
    animation_map.insert(ActionStateCodec::Sprinting, sprinting);

    // Attacks

    let mut attacks = Vec::new();

    let basic_attack_loc = "player/basic_attack";

    let basic_attack_length: u64 = 350;
    let basic_attack_frames: u32 = 7;
    let basic_attack_interval = basic_attack_length / basic_attack_frames as u64;

    let basic_attack_animation = AnimationData::new(
        basic_attack_loc,
        96, 96,
        FrameDataCodec::FixedInterval {
            num_frames: basic_attack_frames as usize,
            interval: basic_attack_interval,
        },
    );

    let basic_attack_stamina_cost = 20;

    let basic_attack_key_frames = vec![
        KeyFrameCodec {
            start_time: 0,
            end_time: basic_attack_length,
            hitbox: HitboxCodec::Static {
                collider: ColliderCodec {
                    format: ColliderCodec::LATEST_FORMAT,
                    collider: ColliderDataCodec::Sphere(1.0)
                },
                offset: (0., 0., 1.).into(),
            },

            health_event: HealthEventKind::Damage(30, DamageKind::Generic),
            disable_on_hit_iframes: None.into(),
            
            exclusion_group: None.into(),
        }
    ];

    let basic_attack = AttackData::new(
        basic_attack_loc,
        basic_attack_length,
        basic_attack_stamina_cost,
        basic_attack_animation,
        format!("{}_particle", basic_attack_loc).as_str(),
        basic_attack_key_frames,
    );
    attacks.push(basic_attack);

    let attack_set = AttackSetData::new(basic_attack_loc, attacks);

    let collider = ColliderDataCodec::Capsule(
        CapsuleCodec::Vertical {
            height: 1.25,
            radius: 0.25,
        }
    );

    let data = CharacterData::new("player", collider)
        .with_animations(animation_map)
        .with_attack_set(attack_set);

    create_character(data)
}