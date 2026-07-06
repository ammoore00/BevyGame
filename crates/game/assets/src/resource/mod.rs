use bevy::prelude::*;

mod audio;
mod ui;
mod font;

pub mod level;
pub mod characters;
pub use {
    audio::{AudioRegistry, AudioResource},
    font::FontBuilder,
    ui::{UiSpriteRegistry, UiSpriteResource}
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        audio::plugin,
        characters::plugin,
        font::plugin,
        level::plugin,
    ));
}