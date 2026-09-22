use crate::codec::{TranslationCodec, TranslationList};
use crate::loader::{LoaderJobManager, RonAssetLoader};
use bevy::prelude::*;
use data::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<TranslationResource>();
    app.init_asset_loader::<RonAssetLoader<TranslationCodec, Translations>>();
}

#[resource_kind(path = "images/ui", asset_kind = Image, file_type = ResourceFileType::Image)]
pub struct UiSpriteResource;

#[resource_kind(path = "text", asset_kind = (), file_type = ResourceFileType::None)]
pub struct TextResource;

#[resource_kind(path = "translations", asset_kind = Translations)]
pub struct TranslationResource;

#[derive(Asset, Clone, TypePath)]
pub struct Translations {
    ui: TranslationList<TextResource>,
}
impl From<TranslationCodec> for Translations {
    fn from(codec: TranslationCodec) -> Self {
        Translations { ui: codec.ui }
    }
}
