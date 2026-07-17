use crate::debug_options::options::global_debug;
use crate::debug_options::DebugOptionsWindowOpen;
use crate::window;
use bevy::prelude::*;
use common::marker;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(DebugOptionsWindowOpen(true)), spawn_debug_window);
}

marker!(DebugOptionsWindow);

fn spawn_debug_window(mut commands: Commands) {
    commands.spawn_scene(debug_options_window_scene());
    commands.trigger(DebugWindowInitialized);
}

fn debug_options_window_scene() -> impl Scene {
    bsn! [
        #DebugOptionsWindow
        DebugOptionsWindow
        window()
        Node {
            position_type: PositionType::Relative,
            width: percent(30),
            padding: UiRect::all(px(16)),
        }
        DespawnOnExit<DebugOptionsWindowOpen>(DebugOptionsWindowOpen(true))
        Children [
            global_debug()
        ]
    ]
}

#[derive(Event)]
pub(super) struct DebugWindowInitialized;