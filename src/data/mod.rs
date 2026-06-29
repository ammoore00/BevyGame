use bevy::prelude::App;

pub mod loader;
pub mod prototyping;
pub mod registry;
pub mod sprite;
pub mod loc;
pub mod resource;

pub mod prelude {
    pub use crate::{
        data::{
            loader::LoaderJobManager,
            loc::{
                loc,
                ResourceLocation
            },
            prototyping::{
                Prototype, PrototypeBuilder, PrototypeFinalizedMarker, PrototypeMarkerToken,
            },
            registry::{ResourceRegistry, SystemRegistry, SystemRegistryMut},
            resource::{
                ResourceFileType,
                ResourceKind,
            }
        },
        define_data_resource, define_resource, define_sprite_resource,
    };
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((loader::plugin,));
}