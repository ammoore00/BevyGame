use std::any::TypeId;
use std::collections::HashMap;
use crate::{data, define_resource};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use maybe_fields_macro::maybe_fields;
use crate::data::loader::{Maybe, RonAssetLoader};
use crate::data::{ResourceFileType, ResourceLocation};
use crate::data::registry::{ResolvedResourceRegistry, ResourceRegistry};
use crate::datagen_api::animation::{AnimationResource, ResolvedAnimationData};
use crate::game::character::attack::{AttackDefinition, AttackResource};
use crate::game::character::state::action_states::{Attacking, Idle, Running, Sprinting, Walking, DEFAULT_STATES, DEFAULT_STATES_NON_ATTACKING};
use crate::game::character::state::state_transitions::ActionStateCapabilities;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<CharacterData>();
    app.init_asset_loader::<RonAssetLoader::<CharacterCodec, CharacterData>>();
}

#[derive(Debug, Clone, Asset, TypePath, derive_new::new)]
pub struct CharacterData {
    name: String,
    state_capabilities: ActionStateCapabilities,
    animations: HashMap<TypeId, ResourceLocation<AnimationResource>>,
    _attacks: Vec<ResourceLocation<AttackResource>>
}
impl CharacterData {
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn state_capabilities(&self) -> &ActionStateCapabilities {
        &self.state_capabilities
    }
    
    pub fn resolve_animation_handles(&self, animation_registry: &ResolvedResourceRegistry<AnimationResource>) -> HashMap<TypeId, Handle<ResolvedAnimationData>> {
        self.animations.iter()
            .map(|(type_id, animation_location)| {
                let animation = animation_registry.get(animation_location)
                    .cloned()
                    // TODO: Replace with non-panic error handling
                    .expect("Failed to retrieve animation handle from registry!");
                (*type_id, animation)
            })
            .collect()
    }
    
    pub fn _resolve_attack_handles(&self, attack_registry: &Res<ResourceRegistry<AttackResource>>) -> Vec<Handle<AttackDefinition>> {
        self._attacks.iter()
            .map(|attack_location| {
                attack_registry.get(attack_location)
                    .cloned()
                    // TODO: Replace with non-panic error handling
                    .expect("Failed to retrieve attack handle from registry!")
            })
            .collect()
    }
}
impl From<CharacterCodec> for CharacterData {
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

        let _attacks = codec.attacks.into_inner().unwrap_or_default();

        CharacterData {
            name: codec.name,
            state_capabilities,
            animations,
            _attacks,
        }
    }
}

#[maybe_fields]
#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct CharacterCodec {
    pub format: u8,
    pub name: String,
    pub allowed_states: Maybe<AllowedStatesCodec>,
    pub animations: HashMap<ActionStateEnum, ResourceLocation<AnimationResource>>,
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

define_resource!(Character, "characters", CharacterData, ResourceFileType::Data);

/// Enum used for referencing action states in data context
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