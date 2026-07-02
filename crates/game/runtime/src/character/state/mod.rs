use bevy::prelude::*;

pub mod update;
pub mod tracking;
// TODO: Split this file into multiple files

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