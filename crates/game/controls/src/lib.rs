use bevy::prelude::*;

mod player;

pub struct ControlsPlugin;
impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(player::plugin);
    }
}
