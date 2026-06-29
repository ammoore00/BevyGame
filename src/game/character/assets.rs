use crate::codec::CharacterCodec;
use crate::codec::ColliderCodec;
use crate::data::prelude::*;
use crate::game::character::attack::{AttackContext, AttackDefinition};
use crate::game::character::state::state_transitions::ActionStateCapabilities;
use crate::{define_data_resource, define_sprite_resource};
use bevy::prelude::*;
use getset::Getters;
use std::any::TypeId;
use std::collections::HashMap;
use crate::data::registry::ResolvedResourceRegistry;
use crate::game::character::animation::ResolvedAnimationData;
use crate::game::character::state::action_states::DEFAULT_STATES;

pub(super) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<CharacterSpriteResource>();
    app.init_asset::<CharacterData>();
    app.init_asset_loader::<RonAssetLoader<CharacterCodec, CharacterData>>();
}

#[derive(Debug, Clone, Asset, TypePath, derive_new::new, Getters)]
pub struct CharacterData {
    state_capabilities: ActionStateCapabilities,
    animations: HashMap<TypeId, ResourceLocation<AnimationResource>>,
    _attack_set: Option<ResourceLocation<AttackSetResource>>,
    #[getset(get = "pub")]
    collider: ColliderCodec,
}
impl CharacterData {
    pub fn state_capabilities(&self) -> &ActionStateCapabilities {
        &self.state_capabilities
    }
    
    pub fn resolve_animation_handles(&self, animation_registry: &ResolvedResourceRegistry<AnimationResource>) -> HashMap<TypeId, Handle<ResolvedAnimationData>> {
        let mut animation_handles = HashMap::new();

        for (state_id, animation_loc) in self.animations.iter() {
            let Some(animation) = animation_registry.get(animation_loc) else {
                error!("Failed to retrieve animation: {}", animation_loc);
                continue;
            };
            animation_handles.insert(*state_id, animation.clone());
        }
        animation_handles
    }
    
    pub fn _resolve_attack_handles(&self, context: &AttackContext) -> Vec<Handle<AttackDefinition>> {
        match &self._attack_set {
            None => Vec::new(),
            Some(attack_set_loc) => {
                let mut attacks = Vec::new();

                let Some(attack_set) = context.attack_set_registry.get_asset(attack_set_loc) else {
                    error!("Failed to retrieve attack_set: {}", attack_set_loc);
                    return attacks;
                };

                for attack_loc in attack_set.iter() {
                    let Some(attack) = context.attack_registry.get_handle(attack_loc) else {
                        error!("Failed to retrieve attack definition: {}", attack_loc);
                        continue;
                    };
                    attacks.push(attack.clone());
                }
                attacks
            }
        }
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

        let _attack_set = codec.attack_set.into_inner();

        CharacterData {
            state_capabilities,
            animations,
            _attack_set,
            collider: codec.collider,
        }
    }
}

define_data_resource!(Character, "characters/characters", CharacterData);
define_sprite_resource!(Character, "characters");