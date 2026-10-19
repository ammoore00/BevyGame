mod command_window;
mod debug_options;
mod inspector;

use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use widgets::background::UiBackgroundStyle;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            command_window::plugin,
            debug_options::plugin,
            inspector::plugin,
        ));
    }
}

fn window() -> impl Scene {
    bsn! {
        @widgets::background::ui_root()
        @widgets::background::ui_background(UiBackgroundStyle::Transparent)
        GlobalZIndex(100)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DebugPickable(pub Uuid);

impl DebugPickable {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
