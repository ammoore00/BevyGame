use crate::characters::{AnimationData, AttackData};
use assets::codec::{
    ColliderCodec, ColliderDataCodec, DamageKind, FrameDataCodec, HealthEventKind, HitboxCodec,
    KeyFrameCodec,
};

pub fn basic_attack(basic_attack_loc: &str, basic_attack_image: &str) -> AttackData {
    let first_frame = 50;
    let second_frame = 200;
    let attack_frame = 50;

    let start_interval = first_frame + second_frame;

    let intervals = vec![
        first_frame,
        second_frame,
        attack_frame,
        attack_frame,
        attack_frame,
        attack_frame,
        attack_frame,
    ];

    let basic_attack_length: u64 = intervals.iter().sum();

    let basic_attack_animation = AnimationData::new(
        basic_attack_image,
        96,
        96,
        FrameDataCodec::Distinct {
            intervals: intervals.clone(),
        },
    );

    let start_time = start_interval;
    let end_time = start_time + attack_frame * 2;

    let basic_attack_key_frames = vec![KeyFrameCodec {
        start_time,
        end_time,
        hitbox: HitboxCodec::Static {
            collider: ColliderCodec {
                format: ColliderCodec::LATEST_FORMAT,
                collider: ColliderDataCodec::Sphere(0.75),
            },
            offset: (0., 0., 1.).into(),
        },

        health_event: HealthEventKind::Damage(30, DamageKind::Generic),
        disable_on_hit_iframes: None.into(),

        exclusion_group: None.into(),
    }];
    
    let basic_attack_stamina_cost = 20;

    AttackData::new(
        basic_attack_loc,
        basic_attack_length,
        basic_attack_stamina_cost,
        basic_attack_animation,
        format!("{}_particle", basic_attack_image).as_str(),
        basic_attack_key_frames,
    )
}
