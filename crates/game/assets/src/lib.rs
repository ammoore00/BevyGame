//! This crate maintains asset loading and static definitions

use bevy::prelude::*;

pub(crate) mod loader;
pub(crate) mod state;

pub mod action_states;
pub mod codec;
pub mod resource;
pub use crate::state::AssetLoadState;

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            action_states::plugin,
            resource::plugin,
            loader::plugin,
            state::plugin,
        ));
    }
}
