use crate::action_states::{Attacking, Idle, Running, Sprinting, Walking, DEFAULT_STATES, DEFAULT_STATES_NON_ATTACKING};
use crate::codec::collider::{CapsuleCodec, ColliderCodec, ColliderKindCodec};
use crate::loader::Maybe;
use crate::resource::character::{AnimationResource, AttackSetResource};
use bevy::prelude::TypePath;
use data::prelude::*;
use maybe_fields::maybe_fields;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;

#[maybe_fields]
#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct CharacterCodec {
    pub format: u8,
    pub allowed_states: Maybe<AllowedStatesCodec>,
    pub animations: HashMap<ActionStateCodec, ResourceLocation<AnimationResource>>,
    pub attack_set: Maybe<ResourceLocation<AttackSetResource>>,
    pub collider: ColliderCodec,
}

impl CharacterCodec {
    pub const LATEST_FORMAT: u8 = 1;
}

impl Default for CharacterCodec {
    fn default() -> Self {
        Self {
            format: Self::LATEST_FORMAT,
            allowed_states: Maybe(None),
            animations: HashMap::new(),
            attack_set: Maybe(None),
            collider: ColliderCodec {
                format: ColliderCodec::LATEST_FORMAT,
                collider: ColliderKindCodec::Capsule(
                    CapsuleCodec::Vertical {
                        radius: 1.25,
                        height: 0.25,
                    }
                )
            },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TypePath)]
pub enum AllowedStatesCodec {
    #[default]
    Default,
    Passive,
    #[serde(untagged)]
    Custom(Vec<ActionStateCodec>),
}

impl AllowedStatesCodec {
    pub fn into_type_ids(self) -> Vec<TypeId> {
        match self {
            AllowedStatesCodec::Default => DEFAULT_STATES.clone(),
            AllowedStatesCodec::Passive => DEFAULT_STATES_NON_ATTACKING.clone(),
            AllowedStatesCodec::Custom(states) => states.into_iter().map(|state| state.into_type_id()).collect(),
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