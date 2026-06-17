use bevy::prelude::*;

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