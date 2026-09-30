use bevy::prelude::*;

mod audio;
mod font;
mod ui;

pub mod characters;
pub mod level;
pub use {
    audio::{AudioRegistry, AudioResource},
    font::FontBuilder,
    ui::{
        CurrentTranslation, TranslationRegistry, TranslationResource, Translations, Translator,
        UiSpriteRegistry, UiSpriteResource, UiTextResource,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        audio::plugin,
        characters::plugin,
        font::plugin,
        level::plugin,
        ui::plugin,
    ));
}
