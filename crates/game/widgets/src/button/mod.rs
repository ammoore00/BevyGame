use bevy::prelude::*;

mod builders;
mod scene;
mod style;

pub use {builders::*, scene::ButtonImpl, style::ButtonStyle};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((style::plugin,));
}
