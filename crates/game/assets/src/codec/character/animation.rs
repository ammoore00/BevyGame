use crate::codec::ActionStateCodec;
use crate::resource::AnimationResource;
use crate::resource::characters::AttackResource;
use bevy::prelude::TypePath;
use common::Facing;
use data::loc::ResourceLocation;
use enum_map::EnumMap;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Default, Debug, Clone, Serialize, Deserialize, TypePath, derive_new::new)]
pub struct CharacterAnimationListCodec {
    pub standard_animations: HashMap<ActionStateCodec, CharacterAnimationSetCodec>,
    pub attack_animations: HashMap<ResourceLocation<AttackResource>, CharacterAnimationSetCodec>,
}

#[derive(Debug, Clone, TypePath)]
pub struct CharacterAnimationSetCodec(pub EnumMap<Facing, ResourceLocation<AnimationResource>>);
impl Serialize for CharacterAnimationSetCodec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let as_hash_map: HashMap<Facing, &ResourceLocation<AnimationResource>> = Facing::iter()
            .map(|facing| (facing, &self.0[facing]))
            .collect();

        as_hash_map.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CharacterAnimationSetCodec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut as_hash_map =
            HashMap::<Facing, ResourceLocation<AnimationResource>>::deserialize(deserializer)?;

        for facing in Facing::iter() {
            if !as_hash_map.contains_key(&facing) {
                return Err(D::Error::custom(format!(
                    "missing animation entry for `{:?}`",
                    facing
                )));
            }
        }

        let map = EnumMap::from_fn(|facing| {
            as_hash_map
                .remove(&facing)
                .expect("all facing animation entries were validated before constructing EnumMap")
        });

        Ok(Self(map))
    }
}
