use bevy::prelude::*;
use bevy::settings::{SaveSettingsSync, SettingsPlugin};
use bevy::window::WindowCloseRequested;

pub const APP_DOMAIN: &str = "com.theladydawn.game";

pub struct GameSettingsPlugin;
impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SettingsPlugin::new(APP_DOMAIN));
        app.add_systems(Update, on_window_close);
    }
}

/// Save settings on window close
fn on_window_close(mut close: MessageReader<WindowCloseRequested>, mut commands: Commands) {
    if let Some(_close_event) = close.read().next() {
        commands.queue(SaveSettingsSync::IfChanged);
        // TODO: Move this to a different system in case there end up being multiple
        //  systems that need to run on app exit
        commands.write_message(AppExit::Success);
    }
}