use crate::action_states::{ActionStateCapabilities, DEFAULT_STATES};
use crate::codec::{CharacterCodec, ColliderCodec};
use crate::loader::{LoaderJobManager, RonAssetLoader};
use crate::resource::characters::{AnimationResource, AttackSetResource};
use bevy::prelude::*;
use data::loc::ResourceLocation;
use data::prelude::ResourceFileType;
use data::resource::resource_kind;
use getset::Getters;
use std::any::TypeId;
use std::collections::HashMap;

pub(super) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<CharacterResource>();
    app.init_asset::<CharacterData>();
    app.init_asset_loader::<RonAssetLoader<CharacterCodec, CharacterData>>();

    app.add_registry_with_discovery::<CharacterSpriteResource>();
}

#[derive(Debug, Clone, Asset, TypePath, derive_new::new, Getters)]
pub struct CharacterData {
    state_capabilities: ActionStateCapabilities,
    #[getset(get = "pub")]
    animations: HashMap<TypeId, ResourceLocation<AnimationResource>>,
    _attack_set: Option<ResourceLocation<AttackSetResource>>,
    #[getset(get = "pub")]
    collider: ColliderCodec,
}
impl CharacterData {
    pub fn state_capabilities(&self) -> &ActionStateCapabilities {
        &self.state_capabilities
    }
}
impl From<CharacterCodec> for CharacterData {
    fn from(codec: CharacterCodec) -> Self {
        let animations = codec
            .animations
            .into_iter()
            .map(|(state, animation)| (state.into_type_id(), animation))
            .collect();

        let states = codec
            .allowed_states
            .into_inner()
            .map(|allowed_states| allowed_states.into_type_ids())
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

#[resource_kind(path = "characters/characters", asset_kind = CharacterData)]
pub struct CharacterResource;

#[resource_kind(path = "images/characters", asset_kind = Image, file_type = ResourceFileType::Image)]
pub struct CharacterSpriteResource;
