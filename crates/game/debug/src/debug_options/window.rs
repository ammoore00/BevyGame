use bevy::prelude::*;
use common::marker;
use crate::debug_options::{DebugOptionsWindowOpen};
use crate::window;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(DebugOptionsWindowOpen(true)), spawn_debug_options_window.spawn());
}

marker!(DebugOptionsWindow);

fn spawn_debug_options_window() -> impl Scene {
    bsn! [
        #DebugOptionsWindow
        DebugOptionsWindow
        window()
        Node {
            right: percent(70)
        }
        DespawnOnExit<DebugOptionsWindowOpen>(DebugOptionsWindowOpen(true))
    ]
}