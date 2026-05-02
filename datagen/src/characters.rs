use std::collections::HashMap;
use bevy_game_2d::data::{ResourceLocation, ResourceType};
use bevy_game_2d::data::sprite::TextureAtlasCodec;
use bevy_game_2d::datagen_api::animation::{AnimationCodec, AnimationResource, AnimationSpriteResource};
use bevy_game_2d::datagen_api::assets::{ActionStateEnum, AllowedStatesCodec, CharacterCodec, CharacterResource};
use bevy_game_2d::datagen_api::attack::AttackResource;
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

    let data = CharacterData::new("player")
        .with_animations(animation_map);

    create_character(data, animations)
}

fn create_character(
    character_data: CharacterData,
    animations: Vec<AnimationData>,
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
    image: ResourceLocation<AnimationSpriteResource>,
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

    fn from_atlas(
        loc: &str,
        image: &str,
        atlas: TextureAtlasData,
        frames: usize,
        interval: u64,
    ) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            image: image.parse().unwrap(),
            atlas: atlas.into(),
            frames,
            interval,
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

const LATEST_CHARACTER_FORMAT: u8 = 1;
const LATEST_ANIMATION_FORMAT: u8 = 1;
const ANIMATION_ROWS: u32 = 8;