mod window;
mod file_manager;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(
        (
            file_manager::plugin,
            window::plugin,
        )
    );
}