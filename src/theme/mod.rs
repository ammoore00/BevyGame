//! Reusable UI widgets & theming.

// Unused utilities may trigger this lints undesirably.
#![allow(dead_code)]

pub mod interaction;
pub mod palette;
pub mod widget_old;
pub mod widgets;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::{interaction::InteractionPalette, palette as ui_palette, widget_old};
}

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        interaction::plugin,
        widget_old::plugin,
        widgets::plugin,
    ));
}
