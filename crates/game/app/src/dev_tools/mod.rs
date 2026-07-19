use bevy::prelude::*;

mod editor;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((editor::plugin,));
}
