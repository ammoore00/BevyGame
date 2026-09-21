use crate::action_states::{
    Attacking, DEFAULT_STATES, DEFAULT_STATES_NON_ATTACKING, Idle, Running, Sprinting, Walking,
};
use crate::codec::character::health::HealthCodec;
use crate::codec::collider::{CapsuleCodec, ColliderCodec, ColliderDataCodec};
use crate::loader::Maybe;
use crate::resource::characters::{AnimationResource, AttackSetResource};
use bevy::prelude::TypePath;
use data::prelude::*;
use maybe_fields::maybe_fields;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;

pub mod animation;
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
    pub animations: HashMap<ActionStateCodec, ResourceLocation<AnimationResource>>,

    /// The attack set to use for this character. See `AttackSetCodec` for more.
    ///
    /// Optional - defaults to none.
    pub attack_set: Maybe<ResourceLocation<AttackSetResource>>,

    /// AI related parameters. See `AiCodec` for more.
    ///
    /// Optional - defaults to no AI
    pub ai_params: Maybe<AiCodec>,
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
            allowed_states: Maybe(None),
            animations: HashMap::new(),

            attack_set: Maybe(None),

            ai_params: Maybe(Some(AiCodec::default())),
        }
    }
}

#[maybe_fields]
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypePath)]
pub struct AiCodec {}

/// Codec for allowed character action states.
///
/// This is not the same as AI state: see `AiCodec` for AI parameters.
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
            AllowedStatesCodec::Default => DEFAULT_STATES.clone(),
            AllowedStatesCodec::Passive => DEFAULT_STATES_NON_ATTACKING.clone(),
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
}
impl ActionStateCodec {
    pub fn into_type_id(self) -> TypeId {
        match self {
            ActionStateCodec::Idle => TypeId::of::<Idle>(),
            ActionStateCodec::Walking => TypeId::of::<Walking>(),
            ActionStateCodec::Running => TypeId::of::<Running>(),
            ActionStateCodec::Sprinting => TypeId::of::<Sprinting>(),
            ActionStateCodec::Attacking => TypeId::of::<Attacking>(),
        }
    }
}
