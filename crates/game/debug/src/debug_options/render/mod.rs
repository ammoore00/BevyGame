mod helpers;
mod physics;
mod ui;
mod palette;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((physics::plugin, ui::plugin));
}
