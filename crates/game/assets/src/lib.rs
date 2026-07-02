//! This crate maintains asset loading and static definitions

use bevy::prelude::*;

pub(crate) mod state;
pub(crate) mod loader;

pub mod codec;
pub mod action_states;
pub mod resource;
pub use crate::{
    state::{
        AssetLoadState,
    },
};

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