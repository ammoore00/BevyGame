use bevy::prelude::*;

mod player;
pub mod buffer;

pub struct ControlsPlugin;
impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((buffer::plugin, player::plugin));
    }
}
