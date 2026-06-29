use bevy::prelude::*;

pub mod loc;
pub mod prototyping;
pub mod registry;
pub mod resource;
pub mod sprite;

pub mod prelude {
    pub use crate::{
        loc::{ResourceLocation, loc},
        prototyping::{
            Prototype, PrototypeBuilder, PrototypeFinalizedMarker, PrototypeMarkerToken,
        },
        registry::{ResourceRegistry, SystemRegistry, SystemRegistryMut},
        resource::{ResourceFileType, ResourceKind},
        define_data_resource, define_resource, define_sprite_resource,
    };
}