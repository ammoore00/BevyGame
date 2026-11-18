use crate::action_states::{
    ALL_STATES, Attacking, Dodging, Idle, MOVEMENT_STATES, Running, Sprinting, Walking,
};
use crate::codec::character::health::HealthCodec;
use crate::codec::collider::{CapsuleCodec, ColliderCodec, ColliderDataCodec};
use crate::maybe::{DefaultOption, Maybe};
use crate::resource::AnimationResource;
use crate::resource::characters::{AiResource, AttackSetResource};
use bevy::prelude::TypePath;
use common::Facing;
use data::prelude::*;
use enum_map::EnumMap;
use maybe_fields::maybe_fields;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub mod ai;
pub mod attack;
pub mod health;

#[maybe_fields]
#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct CharacterCodec {
    /// Format identifier
    pub format: u8,

    /// Collider for the character. This is usually a capsule but can be anything.
    /// See `ColliderCodec` for more.
    pub collider: ColliderCodec,
    /// Parameters related to health. See `HealthCodec` for more,
    pub health_params: HealthCodec,
    /// Allowed entity states. See `AllowedStatesCodec` for more.
    ///
    /// Optional - defaults to all states allowed.
    pub allowed_states: Maybe<AllowedStatesCodec>,
    /// Map of animations to states. Does not include attack animations.
    pub animations: HashMap<ActionStateCodec, CharacterAnimationSetCodec>,

    /// The attack set to use for this character. See `AttackSetCodec` for more.
    ///
    /// Optional - defaults to none.
    pub attack_set: Maybe<ResourceLocation<AttackSetResource>>,

    /// AI related parameters. See `AiCodec` for more.
    ///
    /// Optional - defaults to no AI
    pub ai_params: DefaultOption<ResourceLocation<AiResource>>,
}
impl CharacterCodec {
    pub const LATEST_FORMAT: u8 = 1;
}
impl Default for CharacterCodec {
    fn default() -> Self {
        Self {
            format: Self::LATEST_FORMAT,

            collider: ColliderCodec {
                format: ColliderCodec::LATEST_FORMAT,
                collider: ColliderDataCodec::Capsule(CapsuleCodec::Vertical {
                    radius: 1.25,
                    height: 0.25,
                }),
            },
            health_params: HealthCodec::default(),
            allowed_states: None.into(),
            animations: HashMap::new(),

            attack_set: None.into(),

            ai_params: Default::default(),
        }
    }
}

/// Codec for allowed character action states.
///
/// This is different from AI state: see `AiCodec` for AI parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypePath)]
pub enum AllowedStatesCodec {
    /// Default to allow all states.
    #[default]
    Default,
    /// Allow all states except attacking.
    Passive,
    #[serde(untagged)]
    /// Custom list of states.
    Custom(Vec<ActionStateCodec>),
}
impl AllowedStatesCodec {
    pub fn into_type_ids(self) -> Vec<TypeId> {
        match self {
            AllowedStatesCodec::Default => ALL_STATES.clone(),
            AllowedStatesCodec::Passive => MOVEMENT_STATES.clone(),
            AllowedStatesCodec::Custom(states) => states
                .into_iter()
                .map(|state| state.into_type_id())
                .collect(),
        }
    }
}

/// Enum used for referencing action states in data context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionStateCodec {
    Idle,
    Walking,
    Running,
    Sprinting,
    Attacking,
    Dodging,
}
impl ActionStateCodec {
    pub fn into_type_id(self) -> TypeId {
        match self {
            ActionStateCodec::Idle => TypeId::of::<Idle>(),
            ActionStateCodec::Walking => TypeId::of::<Walking>(),
            ActionStateCodec::Running => TypeId::of::<Running>(),
            ActionStateCodec::Sprinting => TypeId::of::<Sprinting>(),
            ActionStateCodec::Attacking => TypeId::of::<Attacking>(),
            ActionStateCodec::Dodging => TypeId::of::<Dodging>(),
        }
    }
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
