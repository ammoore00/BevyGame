use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<AssetLoadState>();
    app.configure_sets(
        Startup,
        (AssetSystems::RegisterManifests, AssetSystems::LoadAssets).chain(),
    );

    app.configure_sets(
        OnEnter(AssetLoadState::Resolving),
        (
            AssetSystems::ResolveAssets,
            AssetSystems::PopulateResolvedAssets,
        )
            .chain(),
    );

    app.configure_sets(
        OnEnter(AssetLoadState::Done),
        AssetSystems::PopulateAssetRefs,
    );
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AssetSystems {
    /// Register which assets need to be loaded
    RegisterManifests,
    /// Load the assets themselves based on registered manifests
    /// This should only be used by the asset loader!
    LoadAssets,
    /// Resolve any inter-asset references
    ResolveAssets,
    /// Load resolved assets into resolved registries
    /// This should only be used by the asset loader!
    PopulateResolvedAssets,
    /// Populate asset reference resources
    PopulateAssetRefs,
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum AssetLoadState {
    #[default]
    Loading,
    Resolving,
    Done,
}