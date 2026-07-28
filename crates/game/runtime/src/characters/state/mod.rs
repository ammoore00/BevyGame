use bevy::prelude::*;

mod tracking;
mod update;

pub use tracking::{ActionStateTracker, TrySetStateEvent};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((tracking::plugin, update::plugin));
}

#[macro_export]
macro_rules! action_state_scene {
    ($state:ty) => {
        bsn! [
            ActionStateTracker::new($state)
            $state
        ]
    };
}
