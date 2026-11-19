use crate::action_states::{ALL_STATES, ActionStateCapabilities};
use crate::codec::{CharacterAnimationListCodec, CharacterCodec, ColliderCodec};
use crate::loader::{LoaderJobManager, RonAssetLoader};
use crate::maybe::DefaultOption;
use crate::resource::characters::animation::CharacterAnimationList;
use crate::resource::characters::{AiResource, AttackSetResource};
use crate::resource::AnimationResource;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use data::loc::ResourceLocation;
use data::prelude::{ResourceFileType, ResourceKind, SystemRegistry};
use data::resource::{ResolvableData, ResourceVisitError, resource_kind};
use getset::Getters;

pub(super) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<CharacterResource>();
    app.init_asset::<CharacterData>();
    app.init_asset_loader::<RonAssetLoader<CharacterCodec, CharacterData>>();

    app.add_registry_with_discovery::<CharacterSpriteResource>();
}

#[resource_kind(path = "characters/characters", asset_kind = CharacterData, visit_override = true)]
pub struct CharacterResource;
impl CharacterResource {
    fn visit(
        loc: ResourceLocation<Self>,
        character: CharacterData,
        world: &mut World,
    ) -> Result<<Self as ResourceKind>::AssetKind, ResourceVisitError> {
        info!("Resolving character: {}", loc);
        
        let partial = character.into_partial().ok_or(ResourceVisitError(
            "Character has already been resolved".to_string(),
        ))?;
        
        let mut state = SystemState::<SystemRegistry<AnimationResource>>::new(world);
        let context = state
            .get(world)
            .map_err(|error| ResourceVisitError(error.to_string()))?;
        let animations = CharacterAnimationList::try_from_codec(partial.animations, &context)
            .map_err(|error| ResourceVisitError(error.to_string()))?;

        Ok (CharacterData::Resolved(ResolvedCharacterData {
            state_capabilities: partial.state_capabilities,
            animations,
            collider: partial.collider,
            attack_set: partial.attack_set,
            ai_params: partial.ai_params,
        }))
    }
}

pub type CharacterData = ResolvableData<ResolvedCharacterData, PartialCharacterData>;
impl From<CharacterCodec> for CharacterData {
    fn from(value: CharacterCodec) -> Self {
        Self::Partial(value.into())
    }
}

#[derive(Debug, Clone, Asset, TypePath, derive_new::new, Getters)]
pub struct ResolvedCharacterData {
    #[getset(get = "pub")]
    state_capabilities: ActionStateCapabilities,

    #[getset(get = "pub")]
    animations: CharacterAnimationList,

    #[getset(get = "pub")]
    collider: ColliderCodec,

    attack_set: Option<ResourceLocation<AttackSetResource>>,

    ai_params: DefaultOption<ResourceLocation<AiResource>>,
}
impl ResolvedCharacterData {
    pub fn attack_set(&self) -> Option<&ResourceLocation<AttackSetResource>> {
        self.attack_set.as_ref()
    }

    pub fn ai_params(&self) -> DefaultOption<&ResourceLocation<AiResource>> {
        self.ai_params.as_ref()
    }
}

#[derive(Debug, Clone, Asset, TypePath, derive_new::new, Getters)]
pub struct PartialCharacterData {
    state_capabilities: ActionStateCapabilities,

    animations: CharacterAnimationListCodec,

    collider: ColliderCodec,

    attack_set: Option<ResourceLocation<AttackSetResource>>,

    ai_params: DefaultOption<ResourceLocation<AiResource>>,
}
impl From<CharacterCodec> for PartialCharacterData {
    fn from(codec: CharacterCodec) -> Self {
        let states = codec
            .allowed_states
            .into_inner()
            .map(|allowed_states| allowed_states.into_type_ids())
            .unwrap_or_else(|| ALL_STATES.clone());
        let state_capabilities = ActionStateCapabilities::new(states);

        let attack_set = codec.attack_set.into_inner();
        info!("Loading AI params: {:?}", codec.ai_params);

        PartialCharacterData {
            state_capabilities,
            animations: codec.animations,
            attack_set,
            collider: codec.collider,
            ai_params: codec.ai_params,
        }
    }
}

#[resource_kind(path = "images/characters", asset_kind = Image, file_type = ResourceFileType::Image)]
pub struct CharacterSpriteResource;
