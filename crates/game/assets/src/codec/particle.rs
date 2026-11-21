use crate::resource::{AnimationResource, ParticleSpriteResource};
use bevy::prelude::*;
use data::loc::ResourceLocation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct ParticleCodec {
    pub format: u8,
    pub image: ParticleImageCodec,
}
impl ParticleCodec {
    const LATEST_FORMAT: u8 = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub enum ParticleImageCodec {
    Static(StaticParticleCodec),
    Animated(AnimatedParticleCodec),
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct StaticParticleCodec {
    pub image: ResourceLocation<ParticleSpriteResource>
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AnimatedParticleCodec {
    pub animation: ResourceLocation<AnimationResource>,
}
