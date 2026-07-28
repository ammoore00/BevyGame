pub mod palette;
mod update;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((update::plugin,));
}
