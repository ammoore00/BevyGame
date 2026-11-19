use crate::codec::{CharacterAnimationListCodec, CharacterAnimationSetCodec};
use crate::resource::characters::AttackResource;
use crate::resource::{AnimationData, AnimationResource};
use bevy::asset::Handle;
use bevy::prelude::TypePath;
use common::Facing;
use data::loc::ResourceLocation;
use data::prelude::SystemRegistry;
use enum_map::EnumMap;
use std::any::TypeId;
use std::collections::HashMap;

#[derive(Debug, Clone, TypePath)]
pub struct CharacterAnimationList {
    pub standard_animations: HashMap<TypeId, CharacterAnimationSet>,
    pub attack_animations: HashMap<ResourceLocation<AttackResource>, CharacterAnimationSet>,
}
impl From<CharacterAnimationListCodec> for CharacterAnimationList {
    fn from(value: CharacterAnimationListCodec) -> Self {
        let standard_animations = value
            .standard_animations
            .into_iter()
            .map(|(k, v)| (k.into_type_id(), v.into()))
            .collect();
        let attack_animations = value
            .attack_animations
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        
        Self {
            standard_animations,
            attack_animations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterAnimationSet(EnumMap<Facing, ResourceLocation<AnimationResource>>);
impl CharacterAnimationSet {
    pub fn get(&self, facing: Facing) -> &ResourceLocation<AnimationResource> {
        &self.0[facing]
    }
}
impl From<CharacterAnimationSetCodec> for CharacterAnimationSet {
    fn from(codec: CharacterAnimationSetCodec) -> Self {
        Self(codec.0)
    }
}
