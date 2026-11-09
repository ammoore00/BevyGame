mod camera;
mod animation;

use bevy::prelude::*;

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((animation::plugin, camera::plugin));
    }
}