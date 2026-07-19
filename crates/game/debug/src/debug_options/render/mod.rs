mod helpers;
mod navigation;
mod palette;
mod physics;
mod ui;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((navigation::plugin, physics::plugin, ui::plugin));
}
