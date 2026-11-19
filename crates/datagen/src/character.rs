use crate::sprite::TextureAtlasData;
use crate::{WriteError, write_data};
use assets::DefaultOption;
use assets::codec::{
    AllowedStatesCodec, AnimationCodec, AttackCodec, AttackSetCodec, CharacterAnimationListCodec,
    CharacterAnimationSetCodec, CharacterCodec, ColliderCodec, ColliderDataCodec, FrameDataCodec,
    HealthCodec, KeyFrameCodec, TextureAtlasCodec,
};
use assets::resource::AnimationResource;
use assets::resource::characters::{
    AiResource, AttackResource, AttackSetResource, CharacterResource, CharacterSpriteResource,
};
use common::Facing;
use data::prelude::*;
use enum_map::{enum_map, EnumMap};

pub fn create_character(character_data: CharacterData) -> Result<(), WriteError> {
    let attack_set = character_data.attack_set.clone();

    if let Some(attack_set) = attack_set {
        // Write attack set data to disk
        let loc = attack_set.loc.clone();
        let codec = AttackSetCodec::from(attack_set.clone());
        println!("Writing attack set data to: {}", loc.as_path().display());
        write_data(loc, &codec)?;

        // Write all referenced attacks to disk
        for attack_data in &attack_set.attacks {
            let loc = attack_data.loc.clone();
            let codec = AttackCodec::from(attack_data.clone());
            println!("Writing attack data to: {}", loc.as_path().display());
            write_data(loc, &codec)?;
        }
    }

    // Write the characters data to disk
    let loc = character_data.loc.clone();
    let codec = CharacterCodec::from(character_data);
    println!("Writing characters data to: {}", loc.as_path().display());
    write_data(loc, &codec)?;

    Ok(())
}

#[derive(getset::WithSetters)]
pub struct CharacterData {
    loc: ResourceLocation<CharacterResource>,

    collider: ColliderDataCodec,
    health_params: HealthCodec,
    #[getset(set_with)]
    allowed_states: AllowedStatesCodec,
    #[getset(set_with = "pub")]
    animations: CharacterAnimationListCodec,

    attack_set: Option<AttackSetData>,

    ai_params: DefaultOption<ResourceLocation<AiResource>>,
}
impl CharacterData {
    pub fn new(loc: &str, collider: ColliderDataCodec) -> Self {
        Self {
            loc: loc.parse().unwrap(),

            collider,
            health_params: HealthCodec::default(),
            allowed_states: AllowedStatesCodec::default(),
            animations: CharacterAnimationListCodec::default(),

            attack_set: None,

            ai_params: DefaultOption::Default,
        }
    }

    pub fn with_attack_set(self, attack_set: AttackSetData) -> Self {
        Self {
            attack_set: Some(attack_set),
            ..self
        }
    }

    pub fn with_ai_params(self, ai_params: Option<ResourceLocation<AiResource>>) -> Self {
        Self {
            ai_params: ai_params.into(),
            ..self
        }
    }
}
impl From<CharacterData> for CharacterCodec {
    fn from(value: CharacterData) -> Self {
        let attack_set = value.attack_set.map(|attack_set| attack_set.loc);

        let collider = ColliderCodec {
            format: LATEST_COLLIDER_FORMAT,
            collider: value.collider,
        };

        Self {
            format: LATEST_CHARACTER_FORMAT,

            collider,
            health_params: value.health_params,
            allowed_states: Some(value.allowed_states).into(),
            animations: value.animations,

            attack_set: attack_set.into(),

            ai_params: value.ai_params,
        }
    }
}

pub struct CharacterAnimationSetData(pub EnumMap<Facing, String>);
impl From<CharacterAnimationSetData> for CharacterAnimationSetCodec {
    fn from(value: CharacterAnimationSetData) -> Self {
        CharacterAnimationSetCodec(value.0.map(|facing, loc| loc.parse().unwrap()))
    }
}

#[derive(Debug, Clone)]
pub struct AnimationData {
    pub loc: ResourceLocation<AnimationResource>,
    pub image: ResourceLocation<CharacterSpriteResource>,
    pub atlas: TextureAtlasCodec,
    pub frame_data: FrameDataCodec,
}
impl AnimationData {
    pub fn new(loc: &str, width: u32, height: u32, frame_data: FrameDataCodec) -> Self {
        let atlas = TextureAtlasData::new(
            width,
            height,
            frame_data.num_frames() as u32,
            ANIMATION_ROWS,
        );

        Self {
            loc: loc.parse().unwrap(),
            image: loc.parse().unwrap(),
            atlas: atlas.into(),
            frame_data,
        }
    }

    pub fn with_image(self, loc: &str) -> Self {
        Self {
            image: loc.parse().unwrap(),
            ..self
        }
    }
}
impl From<AnimationData> for AnimationCodec {
    fn from(value: AnimationData) -> Self {
        Self {
            format: LATEST_ANIMATION_FORMAT,
            image: value.image,
            atlas: value.atlas,
            frame_data: value.frame_data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttackData {
    loc: ResourceLocation<AttackResource>,
    duration: u64,
    stamina_cost: u32,
    key_frames: Vec<KeyFrameCodec>,
}
impl From<AttackData> for AttackCodec {
    fn from(value: AttackData) -> Self {
        Self {
            format: LATEST_ATTACK_FORMAT,
            duration: value.duration,
            stamina_cost: value.stamina_cost,
            key_frames: value.key_frames,
        }
    }
}
impl AttackData {
    pub fn new(
        loc: &str,
        duration: u64,
        stamina_cost: u32,
        key_frames: Vec<KeyFrameCodec>,
    ) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            duration,
            stamina_cost,
            key_frames,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttackSetData {
    loc: ResourceLocation<AttackSetResource>,
    attacks: Vec<AttackData>,
}
impl From<AttackSetData> for AttackSetCodec {
    fn from(value: AttackSetData) -> Self {
        let attacks = value.attacks.into_iter().map(|attack| attack.loc).collect();

        Self {
            format: LATEST_ATTACK_SET_FORMAT,
            attacks,
        }
    }
}
impl AttackSetData {
    pub fn new(loc: &str, attacks: Vec<AttackData>) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            attacks,
        }
    }
}

const LATEST_CHARACTER_FORMAT: u8 = 1;
const LATEST_ANIMATION_FORMAT: u8 = 1;
const LATEST_ATTACK_FORMAT: u8 = 1;
const LATEST_ATTACK_SET_FORMAT: u8 = 1;
const LATEST_COLLIDER_FORMAT: u8 = 1;

const ANIMATION_ROWS: u32 = 8;

pub fn write_animation_set(
    loc: &str,
    width: u32,
    height: u32,
    frame_data: FrameDataCodec,
) -> Result<CharacterAnimationSetData, WriteError> {
    write_animation_set_with_image(loc, loc, width, height, frame_data)
}

pub fn write_animation_set_with_image(
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

    map.iter().try_for_each(|(facing, ext)| {
        let frame_data = frame_data.clone_with_offset(facing as usize * num_frames);
        let loc = format!("{}/{}", loc, ext);

        let animation =
            AnimationData::new(loc.as_str(), width, height, frame_data).with_image(image_loc);

        write_data(animation.loc.clone(), &AnimationCodec::from(animation))
    })?;

    Ok(CharacterAnimationSetData(map))
}
