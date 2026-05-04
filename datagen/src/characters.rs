use std::collections::HashMap;
use bevy_game_2d::data::{ResourceLocation, ResourceType};
use bevy_game_2d::data::sprite::TextureAtlasCodec;
use bevy_game_2d::datagen_api::animation::{AnimationCodec, AnimationResource};
use bevy_game_2d::datagen_api::assets::{ActionStateEnum, AllowedStatesCodec, CharacterCodec, CharacterResource, CharacterSpriteResource};
use bevy_game_2d::datagen_api::attack::{AttackCodec, AttackResource, AttackSetCodec, AttackSetResource};
use crate::{create_dir, write_data, WriteError};
use crate::sprite::TextureAtlasData;

pub fn generate_characters() -> Result<(), WriteError> {
    create_dir(CharacterResource::ROOT_DIR)?;
    create_dir(AnimationResource::ROOT_DIR)?;
    create_dir(AttackResource::ROOT_DIR)?;
    create_dir(AttackSetResource::ROOT_DIR)?;

    generate_player()?;

    Ok(())
}

fn generate_player() -> Result<(), WriteError> {
    let mut animation_map = HashMap::new();

    // Standard animations

    let idle = AnimationData::new(
        "player/idle",
        64, 64,
        12,
        150,
    );
    animation_map.insert(ActionStateEnum::Idle, idle);

    let walking = AnimationData::new(
        "player/walking",
        64, 64,
        8,
        50,
    );
    animation_map.insert(ActionStateEnum::Walking, walking);

    let running = AnimationData::new(
        "player/running",
        64, 64,
        8,
        50,
    );
    animation_map.insert(ActionStateEnum::Running, running);

    let sprinting = AnimationData::new(
        "player/sprinting",
        64, 64,
        8,
        35,
    ).with_image("player/running");
    animation_map.insert(ActionStateEnum::Sprinting, sprinting);

    // Attacks

    let mut attacks = Vec::new();

    let basic_attack_loc = "player/basic_attack";

    let basic_attack_length: u64 = 350;
    let basic_attack_frames: u32 = 7;
    let basic_attack_interval = basic_attack_length / basic_attack_frames as u64;

    let basic_attack_animation = AnimationData::new(
        basic_attack_loc,
        96, 96,
        basic_attack_frames,
        basic_attack_interval,
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

    let data = CharacterData::new("player")
        .with_animations(animation_map)
        .with_attack_set(attack_set);

    create_character(data)
}

fn create_character(
    character_data: CharacterData,
) -> Result<(), WriteError> {
    let attack_set = character_data.attack_set.clone();

    let animations = character_data.animations.clone();
    let mut animations_list = animations.into_values().collect::<Vec<_>>();

    if let Some(attack_set) = attack_set {
        // Add attack animations to the list of animations which need to be registered
        let mut attack_animations = attack_set.attacks.iter()
            .map(|attack_data| attack_data.animation.clone())
            .collect::<Vec<_>>();
        animations_list.append(&mut attack_animations);

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

    // Write collected state and attack animations to disk
    for animation_data in animations_list {
        let loc = animation_data.loc.clone();
        let codec = AnimationCodec::from(animation_data.clone());
        println!("Writing animation data to: {}", loc.as_path().display());
        write_data(loc, &codec)?;
    }

    // Write the character data to disk
    let loc = character_data.loc.clone();
    let codec = CharacterCodec::from(character_data);
    println!("Writing character data to: {}", loc.as_path().display());
    write_data(loc, &codec)?;

    Ok(())
}

#[derive(getset::WithSetters)]
struct CharacterData {
    loc: ResourceLocation<CharacterResource>,
    #[getset(set_with)]
    allowed_states: AllowedStatesCodec,
    #[getset(set_with)]
    animations: HashMap<ActionStateEnum, AnimationData>,
    attack_set: Option<AttackSetData>,
}
impl CharacterData {
    fn new(
        loc: &str,
    ) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            allowed_states: AllowedStatesCodec::default(),
            animations: HashMap::new(),
            attack_set: None,
        }
    }

    pub fn with_attack_set(self, attack_set: AttackSetData) -> Self {
        Self {
            attack_set: Some(attack_set),
            ..self
        }
    }
}
impl From<CharacterData> for CharacterCodec {
    fn from(value: CharacterData) -> Self {
        let attack_set = value.attack_set.map(|attack_set| attack_set.loc);
        let animations = value.animations.into_iter()
            .map(|(action, animation)| (action, animation.loc))
            .collect();

        Self {
            format: LATEST_CHARACTER_FORMAT,
            allowed_states: Some(value.allowed_states).into(),
            animations,
            attack_set: attack_set.into(),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct AttackData {
    loc: ResourceLocation<AttackResource>,
    duration: u64,
    stamina_cost: usize,
    animation: AnimationData,
    particle_sprite: ResourceLocation<CharacterSpriteResource>,
}
impl From<AttackData> for AttackCodec {
    fn from(value: AttackData) -> Self {
        Self {
            format: LATEST_ATTACK_FORMAT,
            duration: value.duration,
            stamina_cost: value.stamina_cost,
            animation: value.animation.loc,
            particle_sprite: value.particle_sprite,
        }
    }
}
impl AttackData {
    fn new(
        loc: &str,
        duration: u64,
        stamina_cost: usize,
        animation: AnimationData,
        particle_sprite: &str,
    ) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            duration,
            stamina_cost,
            animation,
            particle_sprite: particle_sprite.parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
struct AttackSetData {
    loc: ResourceLocation<AttackSetResource>,
    attacks: Vec<AttackData>,
}
impl From<AttackSetData> for AttackSetCodec {
    fn from(value: AttackSetData) -> Self {
        let attacks = value.attacks.into_iter()
            .map(|attack| attack.loc)
            .collect();

        Self {
            format: LATEST_ATTACK_SET_FORMAT,
            attacks,
        }
    }
}
impl AttackSetData {
    fn new(loc: &str, attacks: Vec<AttackData>) -> Self {
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
const ANIMATION_ROWS: u32 = 8;