use crate::codec::{AnimatedParticleCodec, ParticleCodec, ParticleImageCodec, StaticParticleCodec};
use crate::loader::{LoaderJobManager, RonAssetLoader};
use crate::resource::AnimationResource;
use bevy::prelude::*;
use data::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<ParticleDefinition>();
    app.init_asset_loader::<RonAssetLoader<ParticleCodec, ParticleDefinition>>();
    app.add_registry_with_discovery::<ParticleResource>();

    app.add_registry_with_discovery::<ParticleSpriteResource>();
}

#[resource_kind(path = "characters/attacks", asset_kind = ParticleDefinition)]
pub struct ParticleResource;

#[resource_kind(path = "images/particles", asset_kind = Image, file_type = ResourceFileType::Image)]
pub struct ParticleSpriteResource;

#[derive(Asset, Debug, Clone, TypePath)]
pub struct ParticleDefinition {
    image: ParticleImage,
}
impl From<ParticleCodec> for ParticleDefinition {
    fn from(codec: ParticleCodec) -> Self {
        Self {
            image: codec.image.into(),
        }
    }
}

#[derive(Debug, Clone, TypePath)]
pub enum ParticleImage {
    Static(StaticParticle),
    Animated(AnimatedParticle),
}
impl From<ParticleImageCodec> for ParticleImage {
    fn from(codec: ParticleImageCodec) -> Self {
        match codec {
            ParticleImageCodec::Static(codec) => Self::Static(codec.into()),
            ParticleImageCodec::Animated(codec) => Self::Animated(codec.into()),
        }
    }
}

#[derive(Debug, Clone, TypePath)]
pub struct StaticParticle {
    pub image: ResourceLocation<ParticleSpriteResource>,
}
impl From<StaticParticleCodec> for StaticParticle {
    fn from(codec: StaticParticleCodec) -> Self {
        Self { image: codec.image }
    }
}

#[derive(Debug, Clone, TypePath)]
pub struct AnimatedParticle {
    pub animation: ResourceLocation<AnimationResource>,
}
impl From<AnimatedParticleCodec> for AnimatedParticle {
    fn from(codec: AnimatedParticleCodec) -> Self {
        Self {
            animation: codec.animation,
        }
    }
}
