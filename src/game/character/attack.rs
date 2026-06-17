use crate::data::loader::{LoaderJobManager, RonAssetLoader};
use crate::data::registry::{ResolvedSystemRegistry, SystemRegistry};
use crate::data::{ResourceFileType, ResourceLocation};
use crate::datagen_api::animation::AnimationResource;
use crate::datagen_api::assets::CharacterSpriteResource;
use crate::define_resource;
use crate::{data, define_data_resource};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<AttackDefinition>();
    app.init_asset_loader::<RonAssetLoader<AttackCodec, AttackDefinition>>();
    app.add_registry_with_discovery::<AttackResource>();

    app.init_asset::<AttackSet>();
    app.init_asset_loader::<RonAssetLoader<AttackSetCodec, AttackSet>>();
    app.add_registry_with_discovery::<AttackSetResource>();
}

#[derive(SystemParam)]
pub struct AttackContext<'w> {
    pub attack_registry: SystemRegistry<'w, AttackResource>,
    pub attack_set_registry: SystemRegistry<'w, AttackSetResource>,
    pub animation_registry: ResolvedSystemRegistry<'w, AnimationResource>,
    pub character_sprite_registry: SystemRegistry<'w, CharacterSpriteResource>,
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
impl AttackCodec {
    pub const LATEST_FORMAT: u8 = 1;
}
impl Default for AttackCodec {
    fn default() -> Self {
        AttackCodec {
            format: AttackCodec::LATEST_FORMAT,
            duration: 150,
            stamina_cost: 0,
            animation: "untitled".parse().unwrap(),
            particle_sprite: "untitled".parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Asset, TypePath, Getters)]
pub struct AttackSet {
    #[getset(get = "pub")]
    attacks: Vec<ResourceLocation<AttackResource>>,
}
impl AttackSet {
    pub fn iter(&self) -> impl Iterator<Item = &ResourceLocation<AttackResource>> {
        self.attacks.iter()
    }
}
impl From<AttackSetCodec> for AttackSet {
    fn from(value: AttackSetCodec) -> Self {
        AttackSet {
            attacks: value.attacks,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AttackSetCodec {
    pub format: u8,
    pub attacks: Vec<ResourceLocation<AttackResource>>,
}

define_data_resource!(Attack, "characters/attacks", AttackDefinition);
define_data_resource!(AttackSet, "characters/attack_sets", AttackSet);