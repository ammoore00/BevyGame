use bevy::prelude::*;
use debug_menu::{debug_options, level_render};

mod debug_menu;
mod editor;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(
        (
            debug_menu::plugin,
            editor::plugin,
        )
    );
}