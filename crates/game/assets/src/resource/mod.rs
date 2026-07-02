use bevy::prelude::*;

mod tile;
mod audio;
mod ui;
mod font;

pub mod map;
pub mod character;
pub use {
    audio::{AudioRegistry, AudioResource},
    font::FontBuilder,
    tile::{
        TileAsset, TileFacing, TileLayout, TileRegistry, TileResource, TileShape,
        TileSpriteRegistry, TileSpriteResource,
    },
    ui::{UiSpriteRegistry, UiSpriteResource}
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        audio::plugin,
        character::plugin,
        font::plugin,
        map::plugin,
        tile::plugin,
    ));
}