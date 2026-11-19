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
impl CharacterAnimationList {
    pub fn try_from_codec(
        codec: CharacterAnimationListCodec,
        context: &SystemRegistry<AnimationResource>,
    ) -> Result<Self, AnimationSetError> {
        let standard_animations = codec
            .standard_animations
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    k.into_type_id(),
                    CharacterAnimationSet::try_from_codec(v, context)?,
                ))
            })
            .collect::<Result<_, _>>()?;
        let attack_animations = codec
            .attack_animations
            .into_iter()
            .map(|(k, v)| Ok((k, CharacterAnimationSet::try_from_codec(v, context)?)))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            standard_animations,
            attack_animations,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterAnimationSet(EnumMap<Facing, Handle<AnimationData>>);
impl CharacterAnimationSet {
    pub fn try_from_codec(
        codec: CharacterAnimationSetCodec,
        context: &SystemRegistry<AnimationResource>,
    ) -> Result<Self, AnimationSetError> {
        let map = codec
            .0
            .into_iter()
            .map(|(facing, loc)| {
                let handle = context
                    .get_handle(&loc)
                    .ok_or(AnimationSetError::FailedToGetHandle)?
                    .clone();
                Ok((facing, handle))
            })
            .collect::<Result<EnumMap<Facing, Handle<AnimationData>>, AnimationSetError>>()?;

        Ok(Self(map))
    }

    pub fn get(&self, facing: Facing) -> Handle<AnimationData> {
        self.0[facing].clone()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AnimationSetError {
    #[error("Failed to get handle for animation data")]
    FailedToGetHandle,
}
