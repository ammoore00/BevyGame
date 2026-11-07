mod camera;

use bevy::prelude::*;

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(camera::plugin);
    }
}