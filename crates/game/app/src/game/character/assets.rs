use crate::codec::CharacterCodec;
use crate::codec::ColliderCodec;
use crate::game::character::animation::{AnimationContext, AnimationData, AnimationResource};
use crate::game::character::attack::{AttackContext, AttackDefinition, AttackSetResource};
use crate::game::character::state::capabilities::ActionStateCapabilities;
use crate::game::character::state::states::DEFAULT_STATES;
use assets::{LoaderJobManager, RonAssetLoader};
use bevy::prelude::*;
use data::prelude::*;
use getset::Getters;
use std::any::TypeId;
use std::collections::HashMap;

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
    
    pub fn resolve_animation_handles(&self, animation_context: &AnimationContext) -> HashMap<TypeId, Handle<AnimationData>> {
        let mut animation_handles = HashMap::new();

        for (state_id, animation_loc) in self.animations.iter() {
            match animation_context.get_handle(animation_loc) {
                Ok(animation) => {
                    animation_handles.insert(*state_id, animation.clone());
                }
                Err(err) => {
                    // TODO: Handle this more gracefully
                    panic!("Failed to retrieve animation: {}", err);
                }
            }
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