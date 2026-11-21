use bevy::prelude::*;
use data::prelude::{ResourceFileType, resource_kind};

mod animation;
mod audio;
mod font;
mod particle;
mod ui;

pub mod characters;
pub mod level;

pub use {
    animation::{
        AnimationData, AnimationRegistry, AnimationResource, FrameData, ResolvedAnimationData,
    },
    audio::{AudioRegistry, AudioResource},
    font::FontBuilder,
    particle::{ParticleDefinition, ParticleResource, ParticleSpriteResource},
    ui::{
        CurrentTranslation, TranslationRegistry, TranslationResource, Translations, Translator,
        UiSpriteRegistry, UiSpriteResource, UiTextResource,
    },
};
use crate::loader::LoaderJobManager;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        audio::plugin,
        characters::plugin,
        font::plugin,
        level::plugin,
        particle::plugin,
        ui::plugin,
    ));
    
    // This creates registry overlap with generic sprite resource kinds
    // However, Bevy is smart enough to deduplicate them because the
    // resource locations resolve to the same file path
    app.add_registry_with_discovery::<SpriteResource>();
}

#[resource_kind(path = "images", asset_kind = Image, file_type = ResourceFileType::Image)]
pub struct SpriteResource;
