use assets::resource::characters::{AttackResource, CharacterAnimationList, CharacterAnimationSet};
use bevy::prelude::*;
use common::Facing;
use data::loc::ResourceLocation;
use getset::Getters;
use std::any::TypeId;
use std::collections::HashMap;

#[derive(Component, Debug, Clone)]
pub struct CharacterAnimationTracker {
    pub default_animation: CharacterAnimationSet,
    pub current_animation: CharacterAnimationSet,
    pub last_facing: Option<Facing>,
}

impl CharacterAnimationTracker {
    pub fn new(default: CharacterAnimationSet) -> Self {
        Self {
            current_animation: default.clone(),
            default_animation: default,
            last_facing: None,
        }
    }
}

#[derive(Component, Debug, Clone, Getters)]
pub struct AnimationStateMap {
    #[getset(get = "pub")]
    standard_animations: HashMap<TypeId, CharacterAnimationSet>,
    #[getset(get = "pub")]
    attack_animations: HashMap<ResourceLocation<AttackResource>, CharacterAnimationSet>,
}
impl AnimationStateMap {
    pub fn new(animation_list: CharacterAnimationList) -> Self {
        Self {
            standard_animations: animation_list.standard_animations,
            attack_animations: animation_list.attack_animations,
        }
    }
}
