use bevy::prelude::*;

mod update;
mod tracking;

pub use tracking::{ActionStateTracker, get_state, is_in_movement_state, TrySetStateEvent};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        tracking::plugin,
        update::plugin,
    ));
}

#[macro_export]
macro_rules! action_state_scene {
    ($state:ty) => {
        bsn! [
            ActionStateTracker {
                type_id: {TypeId::of::<$state>()},
            }
            $state
        ]
    };
}