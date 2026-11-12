use bevy::prelude::*;

mod audio;
mod font;
mod ui;

pub mod characters;
pub mod level;
mod animation;

pub use {
    animation::{
        AnimationData, AnimationRegistry, AnimationResource, FrameData, ResolvedAnimationData,
    },
    audio::{AudioRegistry, AudioResource},
    font::FontBuilder,
    ui::{
        CurrentTranslation, TranslationRegistry, TranslationResource, Translations, Translator,
        UiSpriteRegistry, UiSpriteResource, UiTextResource,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        audio::plugin,
        characters::plugin,
        font::plugin,
        level::plugin,
        ui::plugin,
    ));
}
