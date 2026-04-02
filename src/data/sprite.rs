use bevy::prelude::*;
use crate::data::{ResourceFileType, ResourceType};
use crate::data::loader::LoaderJobManager;
use crate::data::registry::ResourceRegistry;

pub type SpriteRegistry = ResourceRegistry<SpriteResource, SpriteImageAsset>;

pub fn plugin(app: &mut App) {
    app.init_asset::<SpriteImageAsset>();
    app.add_resource_registry::<SpriteResource, SpriteImageAsset>();
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SpriteResource;

impl ResourceType for SpriteResource {
    fn root_dir() -> &'static str {
        "images"
    }

    fn file_type() -> ResourceFileType {
        ResourceFileType::Image
    }
}

#[derive(Debug, Clone, Asset, Reflect)]
pub struct SpriteImageAsset;