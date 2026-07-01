// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

#[cfg(feature = "dev")]
use bevy::feathers::FeathersPlugins;
use bevy::input_focus::directional_navigation::DirectionalNavigationPlugin;
use bevy::{asset::AssetMetaCheck, prelude::*};
use common::CommonPlugin;
use physics::PhysicsPlugin;

mod asset_tracking;
mod audio;
mod codec;
mod data;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod gamepad;
mod menus;
mod screens;
mod theme;

mod prelude {
    pub use crate::{
        data::prelude::*,
        AssetSystems
    };
    pub use bevy::prelude::*;
}

pub mod datagen_api {
    pub use crate::{
        codec::*,
        data::prelude::*,
        game::level::{
            grid::tile::{TileFacing, TileShape},
            map::room::{ConnectionFacing, ConnectionSize, RoomConnection},
        },
    };
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy Game 2d".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            DirectionalNavigationPlugin,
        ));

        // Add other plugins.
        app.add_plugins((
            CommonPlugin,
            PhysicsPlugin,

            asset_tracking::plugin,
            data::plugin,
            audio::plugin,
            game::plugin,
            gamepad::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
        ));

        // Asset loading systems
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

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        #[cfg(feature = "dev")]
        {
            info!("Dev tools enabled");
            app.add_plugins((FeathersPlugins, dev_tools::plugin));
        }
    }
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
enum AssetLoadState {
    #[default]
    Loading,
    Resolving,
    Done,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -10000000.0,
            far: 10000000.0,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

#[macro_export]
macro_rules! marker {
    ($marker:ident) => {
        #[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Default)]
        struct $marker;
    };
    (pub $marker:ident) => {
        #[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Default)]
        pub struct $marker;
    };
}
