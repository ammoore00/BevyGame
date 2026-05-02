use std::collections::HashMap;
use bevy_game_2d::data::{ResourceLocation, ResourceType};
use bevy_game_2d::data::sprite::TextureAtlasCodec;
use bevy_game_2d::datagen_api::animation::{AnimationCodec, AnimationResource};
use bevy_game_2d::datagen_api::assets::{ActionStateEnum, AllowedStatesCodec, CharacterCodec, CharacterResource, CharacterSpriteResource};
use bevy_game_2d::datagen_api::attack::{AttackCodec, AttackResource};
use crate::{create_dir, write_data, WriteError};
use crate::sprite::TextureAtlasData;

pub fn generate_characters() -> Result<(), WriteError> {
    create_dir(CharacterResource::ROOT_DIR)?;
    create_dir(AnimationResource::ROOT_DIR)?;
    create_dir(AttackResource::ROOT_DIR)?;

    generate_player()?;

    Ok(())
}

fn generate_player() -> Result<(), WriteError> {
    let mut animations = Vec::new();
    let mut animation_map = HashMap::new();

    // Standard animations

    let idle = AnimationData::new(
        "player/idle",
        64, 64,
        12,
        150,
    );
    animation_map.insert(ActionStateEnum::Idle, idle.loc.clone());
    animations.push(idle);

    let walking = AnimationData::new(
        "player/walking",
        64, 64,
        8,
        50,
    );
    animation_map.insert(ActionStateEnum::Walking, walking.loc.clone());
    animations.push(walking);

    let running = AnimationData::new(
        "player/running",
        64, 64,
        8,
        50,
    );
    animation_map.insert(ActionStateEnum::Running, running.loc.clone());
    animations.push(running);

    let sprinting = AnimationData::new(
        "player/sprinting",
        64, 64,
        8,
        35,
    ).with_image("player/running");
    animation_map.insert(ActionStateEnum::Sprinting, sprinting.loc.clone());
    animations.push(sprinting);

    // Attacks

    let mut attacks = Vec::new();
    let mut attacks_list = Vec::new();

    let basic_attack_loc = "player/basic_attack";

    let basic_attack_length = 350;
    let basic_attack_frames = 7;
    let basic_attack_interval = (basic_attack_length / basic_attack_frames) as u64;

    let basic_attack_animation = AnimationData::new(
        basic_attack_loc,
        96, 96,
        basic_attack_frames,
        basic_attack_interval,
    );
    animations.push(basic_attack_animation);

    let basic_attack_stamina_cost = 20;

    let basic_attack = AttackData::new(
        basic_attack_loc,
        basic_attack_length as u64,
        basic_attack_stamina_cost,
        basic_attack_loc,
        format!("{}_particle", basic_attack_loc).as_str()
    );
    attacks.push(basic_attack);
    attacks_list.push(basic_attack_loc);

    let attacks_list = attacks_list.iter()
        .map(|attack| attack.parse().unwrap())
        .collect();

    let data = CharacterData::new("player")
        .with_animations(animation_map)
        .with_attacks(attacks_list);

    create_character(data, animations, attacks)
}

fn create_character(
    character_data: CharacterData,
    animations: Vec<AnimationData>,
    attacks: Vec<AttackData>,
) -> Result<(), WriteError> {
    let loc = character_data.loc.clone();
    let codec = CharacterCodec::from(character_data);
    println!("Writing character data to: {}", loc.as_path().display());
    write_data(loc, &codec)?;

    for animation_data in animations {
        let loc = animation_data.loc.clone();
        let codec = AnimationCodec::from(animation_data);
        println!("Writing animation data to: {}", loc.as_path().display());
        write_data(loc, &codec)?;
    }

    for attack_data in attacks {
        let loc = attack_data.loc.clone();
        let codec = AttackCodec::from(attack_data);
        println!("Writing attack data to: {}", loc.as_path().display());
        write_data(loc, &codec)?;
    }

    Ok(())
}

#[derive(getset::WithSetters)]
struct CharacterData {
    loc: ResourceLocation<CharacterResource>,
    #[getset(set_with)]
    allowed_states: AllowedStatesCodec,
    #[getset(set_with)]
    animations: HashMap<ActionStateEnum, ResourceLocation<AnimationResource>>,
    #[getset(set_with)]
    attacks: Vec<ResourceLocation<AttackResource>>,
}
impl CharacterData {
    fn new(
        loc: &str,
    ) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            allowed_states: AllowedStatesCodec::default(),
            animations: HashMap::new(),
            attacks: Vec::new(),
        }
    }
}
impl From<CharacterData> for CharacterCodec {
    fn from(value: CharacterData) -> Self {
        let attacks = if value.attacks.is_empty() {
            None
        } else {
            Some(value.attacks)
        };

        Self {
            format: LATEST_CHARACTER_FORMAT,
            allowed_states: Some(value.allowed_states).into(),
            animations: value.animations,
            attacks: attacks.into(),
        }
    }
}

struct AnimationData {
    loc: ResourceLocation<AnimationResource>,
    image: ResourceLocation<CharacterSpriteResource>,
    atlas: TextureAtlasCodec,
    frames: usize,
    interval: u64,
}
impl AnimationData {
    fn new(
        loc: &str,
        width: u32,
        height: u32,
        num_frames: u32,
        interval: u64,
    ) -> Self {
        let atlas = TextureAtlasData::new(
            width,
            height,
            num_frames,
            ANIMATION_ROWS,
        );

        Self {
            loc: loc.parse().unwrap(),
            image: loc.parse().unwrap(),
            atlas: atlas.into(),
            frames: num_frames as usize,
            interval,
        }
    }

    fn with_image(self, loc: &str) -> Self {
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
            frames: value.frames,
            interval: value.interval,
        }
    }
}

struct AttackData {
    loc: ResourceLocation<AttackResource>,
    pub duration: u64,
    pub stamina_cost: usize,
    pub animation: ResourceLocation<AnimationResource>,
    pub particle_sprite: ResourceLocation<CharacterSpriteResource>,
}
impl From<AttackData> for AttackCodec {
    fn from(value: AttackData) -> Self {
        Self {
            format: LATEST_ATTACK_FORMAT,
            duration: value.duration,
            stamina_cost: value.stamina_cost,
            animation: value.animation,
            particle_sprite: value.particle_sprite,
        }
    }
}
impl AttackData {
    fn new(
        loc: &str,
        duration: u64,
        stamina_cost: usize,
        animation: &str,
        particle_sprite: &str,
    ) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            duration,
            stamina_cost,
            animation: animation.parse().unwrap(),
            particle_sprite: particle_sprite.parse().unwrap(),
        }
    }
}

const LATEST_CHARACTER_FORMAT: u8 = 1;
const LATEST_ANIMATION_FORMAT: u8 = 1;
const LATEST_ATTACK_FORMAT: u8 = 1;
const ANIMATION_ROWS: u32 = 8;