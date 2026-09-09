mod command_window;
mod debug_options;

use bevy::prelude::*;
use common::InputBlocker;
use widgets::background::UiBackgroundStyle;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((command_window::plugin, debug_options::plugin));
    }
}

fn window() -> impl Scene {
    bsn! [
        widgets::background::ui_root()
        widgets::background::ui_background(UiBackgroundStyle::Transparent)
        InputBlocker
        GlobalZIndex(100)
    ]
}
