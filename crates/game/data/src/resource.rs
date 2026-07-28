use bevy::asset::Asset;
use bevy::prelude::Reflect;
use std::fmt::Debug;
use std::hash::Hash;

pub trait ResourceKind:
    Debug + Reflect + Clone + Hash + Eq + Send + Sync + Reflect + 'static
{
    type AssetKind: Asset + Clone + Send + Sync + 'static;
    const ROOT_DIR: &'static str;
    const FILE_TYPE: ResourceFileType;
}

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

#[macro_export]
macro_rules! define_resource {
    ($name:ident, $path:expr, $asset_type:ty, $file_type:expr) => {
        paste::paste! {
            #[allow(unused)]
            pub type [<$name Registry>] = $crate::registry::ResourceRegistry<[<$name Resource>]>;

            #[allow(unused)]
            #[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Default, Reflect)]
            pub struct [<$name Resource>];

            impl $crate::resource::ResourceKind for [<$name Resource>] {
                type AssetKind = $asset_type;

                const ROOT_DIR: &'static str = $path;
                const FILE_TYPE: $crate::resource::ResourceFileType = $file_type;
            }
        }
    };
}

#[macro_export]
macro_rules! define_data_resource {
    ($name:ident, $path:literal, $asset_type:ty) => {
        paste::paste! {
            $crate::define_resource!($name, const_format::concatcp!("data/", $path), $asset_type, $crate::resource::ResourceFileType::Data);
        }
    }
}
