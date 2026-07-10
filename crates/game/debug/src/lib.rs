mod commands;
mod debug_options;

use bevy::prelude::*;
use widgets::background::UiBackgroundStyle;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((commands::plugin, debug_options::plugin));
    }
}

fn window() -> impl Scene {
    bsn! [
        widgets::background::ui_root()
        widgets::background::ui_background(UiBackgroundStyle::Transparent)
        GlobalZIndex(100)
    ]
}