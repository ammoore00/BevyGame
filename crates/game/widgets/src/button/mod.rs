use bevy::prelude::*;

mod builders;
mod style;
mod scene;

pub use {
    builders::*,
    scene::{ButtonImpl,},
    style::{ButtonStyle,}
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((style::plugin,));
}