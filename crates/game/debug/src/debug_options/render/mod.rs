mod entity_data;
mod draw;
mod navigation;
mod palette;
mod physics;
mod ui;
mod label;
mod ai;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ai::plugin,
        entity_data::plugin,
        label::plugin,
        navigation::plugin,
        physics::plugin,
        ui::plugin,
    ));
}
