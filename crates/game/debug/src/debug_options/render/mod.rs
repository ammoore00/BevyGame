mod health;
mod draw;
mod navigation;
mod palette;
mod physics;
mod ui;
mod label;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        health::plugin,
        navigation::plugin,
        physics::plugin,
        ui::plugin,
    ));
}
