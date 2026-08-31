use bevy::prelude::*;
use data::prelude::ResourceFileType;
use data::resource::resource_kind;

#[resource_kind(path = "images/ui", asset_kind = Image, file_type = ResourceFileType::Image)]
pub struct UiSpriteResource;
