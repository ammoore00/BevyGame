mod player;

use crate::characters::{AnimationData, AttackData, CharacterAnimationSetData};
use crate::defs::characters::player::generate_player;
use crate::{WriteError, create_dir, write_data};
use assets::codec::{AnimationCodec, ColliderCodec, ColliderDataCodec, DamageKind, DistinctCodec, FrameDataCodec, HealthEventKind, HitboxCodec, KeyFrameCodec};
use assets::resource::AnimationResource;
use assets::resource::characters::{
    AiResource, AttackResource, AttackSetResource, CharacterResource,
};
use common::Facing;
use data::prelude::ResourceKind;
use enum_map::enum_map;

pub fn generate_characters() -> Result<(), WriteError> {
    create_dir(CharacterResource::ROOT_DIR)?;
    create_dir(AnimationResource::ROOT_DIR)?;
    create_dir(AttackResource::ROOT_DIR)?;
    create_dir(AttackSetResource::ROOT_DIR)?;
    create_dir(AiResource::ROOT_DIR)?;

    generate_player()?;

    Ok(())
}

fn write_animation_set(
    loc: &str,
    width: u32,
    height: u32,
    frame_data: FrameDataCodec,
) -> Result<CharacterAnimationSetData, WriteError> {
    write_animation_set_with_image(loc, loc, width, height, frame_data)
}

fn write_animation_set_with_image(
    loc: &str,
    image_loc: &str,
    width: u32,
    height: u32,
    frame_data: FrameDataCodec,
) -> Result<CharacterAnimationSetData, WriteError> {
    let num_frames = frame_data.num_frames();
    
    let map = enum_map! {
        Facing::NorthWest => "northwest",
        Facing::North => "north",
        Facing::NorthEast => "northeast",
        Facing::East => "east",
        Facing::SouthEast => "southeast",
        Facing::South => "south",
        Facing::SouthWest => "southwest",
        Facing::West => "west",
    }
        .map(|_, s| format!("{}/{}", loc, s));
    
    map.iter()
        .try_for_each(|(facing, ext)| {
            let frame_data = frame_data.clone_with_offset(facing as usize * num_frames);
            let loc = format!("{}/{}", loc, ext);
            
            let animation = AnimationData::new(
                loc.as_str(),
                width,
                height,
                frame_data,
            ).with_image(image_loc);
            
            write_data(animation.loc.clone(), &AnimationCodec::from(animation))
        })?;
    
    Ok(CharacterAnimationSetData(map))
}

pub fn basic_attack(
    basic_attack_loc: &str,
    delay_second_frame: bool,
) -> AttackData {
    let first_frame = 50;
    let second_frame = if delay_second_frame { 200 } else { 50 };
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
        basic_attack_key_frames,
    )
}
