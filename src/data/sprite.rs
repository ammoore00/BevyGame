use bevy::prelude::*;
use crate::data::{ResourceFileType, ResourceType};
use crate::data::loader::LoaderJobManager;
use crate::data::registry::ResourceRegistry;

pub type SpriteRegistry = ResourceRegistry<SpriteResource>;

pub fn plugin(app: &mut App) {
    app.add_resource_registry::<SpriteResource>();
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct SpriteResource;
impl ResourceType for SpriteResource {
    type AssetType = Image;
    fn root_dir() -> &'static str {
        "images"
    }
    fn file_type() -> ResourceFileType {
        ResourceFileType::Image
    }
}