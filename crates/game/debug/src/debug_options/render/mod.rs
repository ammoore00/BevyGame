mod helpers;
mod physics;
mod ui;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((helpers::plugin, physics::plugin, ui::plugin));
}
