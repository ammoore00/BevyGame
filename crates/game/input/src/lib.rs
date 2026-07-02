use bevy::prelude::*;

pub mod gamepad;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((gamepad::plugin,));
    }
}