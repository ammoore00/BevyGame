pub mod animation;

pub use bevy::prelude::*;

pub struct RenderBackendPlugin;
impl Plugin for RenderBackendPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(animation::plugin);
    }
}
