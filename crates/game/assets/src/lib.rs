mod state;
mod loader;

use bevy::prelude::*;

pub use crate::{
    state::{
        AssetLoadState,
        AssetSystems,
    },
    loader::{
        LoaderJobManager,
        // TODO: All uses of this type will be moved into this module, so this export should be removed then
        Maybe,
        RonAssetLoader,
    },
};

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            loader::plugin,
            state::plugin,
        ));
    }
}