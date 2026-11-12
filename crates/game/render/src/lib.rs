mod camera;

use bevy::prelude::*;
use render_api::RenderBackendPlugin;

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenderBackendPlugin);
        app.add_plugins(camera::plugin);
    }
}
