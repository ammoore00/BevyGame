mod file_manager;
mod window;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((file_manager::plugin, window::plugin));
}
