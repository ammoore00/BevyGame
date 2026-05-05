use bevy::prelude::*;

mod debug_options;
mod debug_menu;
mod level_render;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(
        (
            debug_options::plugin,
            debug_menu::plugin,
            level_render::plugin,
        )
    );
}