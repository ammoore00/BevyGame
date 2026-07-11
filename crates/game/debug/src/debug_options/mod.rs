mod options;
mod window;

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((options::plugin, window::plugin,));

    app.init_state::<DebugOptionsWindowOpen>();

    app.add_systems(
        Update,
        (
            set_debug_options_window_open.run_if(in_state(DebugOptionsWindowOpen(false))),
            set_debug_options_window_closed.run_if(in_state(DebugOptionsWindowOpen(true))),
        )
            .run_if(input_just_pressed(KeyCode::F1)),
    );
}

fn set_debug_options_window_open(mut state: ResMut<NextState<DebugOptionsWindowOpen>>) {
    state.set(DebugOptionsWindowOpen(true));
}

fn set_debug_options_window_closed(mut state: ResMut<NextState<DebugOptionsWindowOpen>>) {
    state.set(DebugOptionsWindowOpen(false));
}

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct DebugOptionsWindowOpen(bool);
