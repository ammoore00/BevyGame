mod window;

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(window::plugin);

    app.init_state::<InspectorWindowOpen>();
}

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct InspectorWindowOpen(bool);
