use crate::WriteError;
use crate::character::{
    AttackSetData, CharacterData, create_character, write_animation_set,
    write_animation_set_with_image,
};
use crate::defs::characters::basic_attack;
use assets::codec::{
    ActionStateCodec, CapsuleCodec, CharacterAnimationListCodec, ColliderDataCodec, DistinctCodec,
    FixedIntervalCodec, FrameDataCodec,
};
use std::collections::HashMap;

pub fn generate_player() -> Result<(), WriteError> {
    let mut animation_map = HashMap::new();

    // Standard animations

    let fixed = |num_frames: usize, interval: u64| {
        FrameDataCodec::FixedInterval(FixedIntervalCodec::new(num_frames, interval))
    };

    let idle = write_animation_set("player/idle", 64, 64, fixed(12, 150))?;
    animation_map.insert(ActionStateCodec::Idle, idle.into());

    let walking = write_animation_set("player/walking", 64, 64, fixed(8, 50))?;
    animation_map.insert(ActionStateCodec::Walking, walking.into());

    let running = write_animation_set("player/running", 64, 64, fixed(8, 50))?;
    animation_map.insert(ActionStateCodec::Running, running.into());

    let sprinting =
        write_animation_set_with_image("player/sprinting", "player/running", 64, 64, fixed(8, 35))?;
    animation_map.insert(ActionStateCodec::Sprinting, sprinting.into());

    let dodging = write_animation_set("player/rolling", 48, 48, fixed(11, 35))?;
    animation_map.insert(ActionStateCodec::Dodging, dodging.into());

    // Attacks

    let mut attacks = Vec::new();

    let basic_attack_loc = "player/basic_attack";

    let basic_attack = basic_attack(basic_attack_loc, false);
    attacks.push(basic_attack);

    let basic_attack_image = "player/basic_attack";
    let intervals = vec![50; 8];

    let basic_attack_animation = write_animation_set_with_image(
        basic_attack_loc,
        basic_attack_image,
        96,
        96,
        FrameDataCodec::Distinct(DistinctCodec::new(intervals)),
    )?;

    let mut attack_animation_map = HashMap::new();
    attack_animation_map.insert(
        basic_attack_loc.parse().unwrap(),
        basic_attack_animation.into(),
    );

    let attack_set = AttackSetData::new(basic_attack_loc, attacks);

    // Finish

    let collider = ColliderDataCodec::Capsule(CapsuleCodec::Vertical {
        height: 1.25,
        radius: 0.25,
    });

    let animation_list = CharacterAnimationListCodec::new(animation_map, attack_animation_map);

    let data = CharacterData::new("player", collider)
        .with_animations(animation_list)
        .with_attack_set(attack_set)
        .with_ai_params(None);

    create_character(data)?;

    Ok(())
}
