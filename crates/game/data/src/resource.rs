use bevy::asset::Asset;
use bevy::prelude::{Reflect, World};
use std::fmt::Debug;
use std::hash::Hash;

use crate::loc::ResourceLocation;
pub use define_resource::resource_kind;

pub trait ResourceKind:
    Debug + Reflect + Clone + Hash + Eq + Send + Sync + Reflect + 'static
{
    type AssetKind: Asset + Clone + Send + Sync + 'static;
    const ROOT_DIR: &'static str;
    const FILE_TYPE: ResourceFileType;

    /// Perform any post-processing on the resource after loading has finished
    fn visit(
        _loc: ResourceLocation<Self>,
        asset: Self::AssetKind,
        _world: &mut World,
    ) -> Result<Self::AssetKind, ResourceVisitError> {
        Ok(asset)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Failed to visit resource: {0}")]
pub struct ResourceVisitError(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceFileType {
    Image,
    Audio,
    Font,
    Data,
    Other(&'static str),
}

impl ResourceFileType {
    pub const fn ext(self) -> &'static str {
        match self {
            ResourceFileType::Image => "png",
            ResourceFileType::Audio => "ogg",
            ResourceFileType::Font => "ttf",
            ResourceFileType::Data => "ron",
            ResourceFileType::Other(s) => s,
        }
    }
}
