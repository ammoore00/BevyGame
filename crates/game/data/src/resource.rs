use crate::loc::ResourceLocation;
use bevy::asset::Asset;
use bevy::prelude::{Reflect, TypePath, World};
use std::fmt::Debug;
use std::hash::Hash;

pub use define_resource::resource_kind;

pub trait ResourceKind: Debug + Reflect + Clone + Hash + Eq + Send + Sync + 'static {
    type AssetKind: Asset + Clone + Send + Sync + 'static;
    const ROOT_DIR: &'static str;
    const FILE_TYPE: ResourceFileType;

    /// Perform any post-processing on the resource after loading has finished.
    ///
    /// `visit` is intended for game-defined data assets that require dependency
    /// resolution. It should remain a no-op for Bevy-native asset types such as
    /// `Image`, because multiple resource kinds may intentionally alias the same
    /// underlying asset path.
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
    /// Used to represent resource locations with no associated file
    None,
}

impl ResourceFileType {
    pub const fn ext(self) -> &'static str {
        match self {
            ResourceFileType::Image => "png",
            ResourceFileType::Audio => "ogg",
            ResourceFileType::Font => "ttf",
            ResourceFileType::Data => "ron",
            ResourceFileType::Other(s) => s,
            ResourceFileType::None => "",
        }
    }
}

#[derive(Asset, Debug, Clone, PartialEq, TypePath)]
pub enum ResolvableData<Resolved, Partial>
where
    Resolved: TypePath + Send + Sync + 'static,
    Partial: TypePath + Send + Sync + 'static,
{
    Resolved(Resolved),
    Partial(Partial),
}
impl<R, P> ResolvableData<R, P>
where
    R: TypePath + Send + Sync + 'static,
    P: TypePath + Send + Sync + 'static,
{
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved(_))
    }

    pub fn as_resolved(&self) -> Option<&R> {
        match self {
            Self::Resolved(data) => Some(data),
            Self::Partial(_) => None,
        }
    }

    pub fn as_partial(&self) -> Option<&P> {
        match self {
            Self::Resolved(_) => None,
            Self::Partial(data) => Some(data),
        }
    }
    
    pub fn into_partial(self) -> Option<P> {
        match self {
            Self::Resolved(_) => None,
            Self::Partial(data) => Some(data),
        }
    }

    /// #Panics
    /// If the animation data is not resolved
    pub fn resolved(&self) -> &R {
        self.as_resolved()
            .expect("Cannot unwrap unresolved data!")
    }
}
