//! Development tools for the game. This plugin is only enabled in dev builds.

mod debug_options;
mod physics;
mod debug_menu;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(
        (
            debug_options::plugin,
            debug_menu::plugin,
            physics::plugin,
        )
    );
}