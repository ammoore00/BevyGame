pub mod loc;
pub mod prototyping;
pub mod registry;
pub mod resource;
pub mod sprite;

pub mod prelude {
    pub use crate::{
        loc::{ResourceLocation, loc, ResourceLoc},
        prototyping::{
            Prototype, PrototypeBuilder, PrototypeFinalizedMarker, PrototypeMarkerToken,
        },
        registry::{ResourceRegistry, SystemRegistry, SystemRegistryMut},
        resource::{ResourceFileType, ResourceKind, resource_kind},
    };
}
