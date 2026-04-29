use std::any::{Any, TypeId};
use std::collections::HashMap;
use crate::data;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use maybe_fields_macro::maybe_fields;
use crate::data::loader::{Maybe, InlineOrResourceLocation, RonAssetLoader};
use crate::data::{ResourceFileType, ResourceLocation};
use crate::datagen_api::animation::{AnimationCodec, AnimationResource};
use crate::define_resource;
use crate::game::character::attack::{AttackCodec, AttackResource};
use crate::game::character::state::action_states::{Attacking, Idle, Running, Sprinting, Walking, DEFAULT_STATES, DEFAULT_STATES_NON_ATTACKING};
use crate::game::character::state::state_transitions::ActionStateCapabilities;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<CharacterAsset>();
    app.init_asset_loader::<RonAssetLoader::<CharacterCodec, CharacterAsset>>();
}

/// Enum used for referencing action states in data context
/// Attacking is not present here as it has its own special handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionStateEnum {
    Idle,
    Walking,
    Running,
    Sprinting,
    Attacking,
}
impl ActionStateEnum {
    pub fn into_type_id(self) -> TypeId {
        match self {
            ActionStateEnum::Idle => TypeId::of::<Idle>(),
            ActionStateEnum::Walking => TypeId::of::<Walking>(),
            ActionStateEnum::Running => TypeId::of::<Running>(),
            ActionStateEnum::Sprinting => TypeId::of::<Sprinting>(),
            ActionStateEnum::Attacking => TypeId::of::<Attacking>(),
        }
    }
}

#[derive(Debug, Clone, Asset, TypePath)]
pub struct CharacterAsset {
    name: String,
    state_capabilities: ActionStateCapabilities,
    animations: HashMap<TypeId, InlineOrResourceLocation<AnimationResource, AnimationCodec>>,
    attacks: Vec<ResourceLocation<AttackResource>>
}
impl From<CharacterCodec> for CharacterAsset {
    fn from(codec: CharacterCodec) -> Self {
        let animations = codec.animations.into_iter()
            .map(|(state, animation)| {
                (state.into_type_id(), animation)
            })
            .collect();

        let states = codec.allowed_states.into_inner()
            .map(|allowed_states| {
                allowed_states.into_type_ids()
            })
            .unwrap_or_else(|| DEFAULT_STATES.clone());
        let state_capabilities = ActionStateCapabilities::new(states);

        let attacks = codec.attacks.into_inner().unwrap_or_default();

        CharacterAsset {
            name: codec.name,
            state_capabilities,
            animations,
            attacks,
        }
    }
}

#[maybe_fields]
#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct CharacterCodec {
    pub format: u8,
    pub name: String,
    pub allowed_states: Maybe<AllowedStatesCodec>,
    pub animations: HashMap<ActionStateEnum, InlineOrResourceLocation<AnimationResource, AnimationCodec>>,
    pub attacks: Maybe<Vec<ResourceLocation<AttackResource>>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TypePath)]
pub enum AllowedStatesCodec {
    #[default]
    Default,
    Passive,
    #[serde(untagged)]
    Custom(Vec<ActionStateEnum>),
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

define_resource!(Character, "characters", CharacterAsset, ResourceFileType::Data);