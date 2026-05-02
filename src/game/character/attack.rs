use std::time::Duration;
use bevy::ecs::system::SystemParam;
use crate::data;
use bevy::prelude::*;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use crate::data::{ResourceFileType, ResourceLocation};
use crate::data::loader::{LoaderJobManager, RonAssetLoader};
use crate::data::registry::{ResolvedSystemRegistry, SystemRegistry};
use crate::datagen_api::animation::AnimationResource;
use crate::datagen_api::assets::CharacterSpriteResource;
use crate::define_resource;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<AttackDefinition>();
    app.init_asset_loader::<RonAssetLoader<AttackCodec, AttackDefinition>>();
    app.add_registry_with_discovery::<AttackResource>();
}

#[derive(Debug, Clone, Asset, TypePath, Getters, CopyGetters)]
pub struct AttackDefinition {
    #[getset(get = "pub")]
    duration: Duration,
    #[getset(get_copy = "pub")]
    stamina_cost: usize,
    #[getset(get = "pub")]
    animation: ResourceLocation<AnimationResource>,
    #[getset(get = "pub")]
    particle_sprite: ResourceLocation<CharacterSpriteResource>,
}
impl From<AttackCodec> for AttackDefinition {
    fn from(value: AttackCodec) -> Self {
        AttackDefinition {
            duration: Duration::from_millis(value.duration),
            stamina_cost: value.stamina_cost,
            animation: value.animation,
            particle_sprite: value.particle_sprite,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AttackCodec {
    pub format: u8,
    pub duration: u64,
    pub stamina_cost: usize,
    pub animation: ResourceLocation<AnimationResource>,
    pub particle_sprite: ResourceLocation<CharacterSpriteResource>,
}

define_resource!(Attack, "attacks", AttackDefinition, ResourceFileType::Data);

#[derive(SystemParam)]
pub struct AttackContext<'w> {
    pub attack_registry: SystemRegistry<'w, AttackResource>,
    pub animation_registry: ResolvedSystemRegistry<'w, AnimationResource>,
    pub character_sprite_registry: SystemRegistry<'w, CharacterSpriteResource>,
}