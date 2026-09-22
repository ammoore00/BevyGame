use bevy::prelude::*;
use data::prelude::ResourceFileType;
use data::resource::resource_kind;

pub(super) fn plugin(app: &mut App) {
    
}

#[resource_kind(path = "images/ui", asset_kind = Image, file_type = ResourceFileType::Image)]
pub struct UiSpriteResource;

#[resource_kind(path = "text", asset_kind = (), file_type = ResourceFileType::None)]
pub struct TextResource;

#[resource_kind(path = "translations", asset_kind = Translation)]
pub struct TranslationResource;

#[derive(Asset, Clone, TypePath)]
pub struct Translation;