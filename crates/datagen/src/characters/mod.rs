mod player;
mod test_npc;

use crate::characters::player::generate_player;
use crate::characters::test_npc::generate_test_npc;
use crate::sprite::TextureAtlasData;
use crate::{create_dir, write_data, WriteError};
use data::prelude::*;
use std::collections::HashMap;
use assets::codec::{ActionStateCodec, AllowedStatesCodec, AnimationCodec, AttackCodec, AttackSetCodec, CharacterCodec, ColliderCodec, ColliderKindCodec, FrameDataCodec, TextureAtlasCodec};
use assets::resource::characters::{AnimationResource, AttackResource, AttackSetResource, CharacterResource, CharacterSpriteResource};

pub fn generate_characters() -> Result<(), WriteError> {
    create_dir(CharacterResource::ROOT_DIR)?;
    create_dir(AnimationResource::ROOT_DIR)?;
    create_dir(AttackResource::ROOT_DIR)?;
    create_dir(AttackSetResource::ROOT_DIR)?;

    generate_player()?;
    generate_test_npc()?;

    Ok(())
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

    // Write the characters data to disk
    let loc = character_data.loc.clone();
    let codec = CharacterCodec::from(character_data);
    println!("Writing characters data to: {}", loc.as_path().display());
    write_data(loc, &codec)?;

    Ok(())
}

#[derive(getset::WithSetters)]
struct CharacterData {
    loc: ResourceLocation<CharacterResource>,
    #[getset(set_with)]
    allowed_states: AllowedStatesCodec,
    #[getset(set_with)]
    animations: HashMap<ActionStateCodec, AnimationData>,
    attack_set: Option<AttackSetData>,
    collider: ColliderKindCodec,
}
impl CharacterData {
    fn new(
        loc: &str,
        collider: ColliderKindCodec,
    ) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            allowed_states: AllowedStatesCodec::default(),
            animations: HashMap::new(),
            attack_set: None,
            collider,
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

        let collider = ColliderCodec {
            format: LATEST_COLLIDER_FORMAT,
            collider: value.collider,
        };

        Self {
            format: LATEST_CHARACTER_FORMAT,
            allowed_states: Some(value.allowed_states).into(),
            animations,
            attack_set: attack_set.into(),
            collider,
        }
    }
}

#[derive(Debug, Clone)]
struct AnimationData {
    loc: ResourceLocation<AnimationResource>,
    image: ResourceLocation<CharacterSpriteResource>,
    atlas: TextureAtlasCodec,
    frame_data: FrameDataCodec,
}
impl AnimationData {
    fn new(
        loc: &str,
        width: u32,
        height: u32,
        frame_data: FrameDataCodec,
    ) -> Self {
        let atlas = TextureAtlasData::new(
            width,
            height,
            frame_data.num_frames(),
            ANIMATION_ROWS,
        );

        Self {
            loc: loc.parse().unwrap(),
            image: loc.parse().unwrap(),
            atlas: atlas.into(),
            frame_data
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
            frame_data: value.frame_data,
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
const LATEST_COLLIDER_FORMAT: u8 = 1;

const ANIMATION_ROWS: u32 = 8;