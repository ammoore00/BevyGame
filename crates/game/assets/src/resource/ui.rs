use crate::codec::{TranslationCodec, TranslationList};
use crate::loader::{LoaderJobManager, RonAssetLoader};
use bevy::prelude::*;
use data::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<Translations>();
    app.add_registry_with_discovery::<TranslationResource>();
    app.init_asset_loader::<RonAssetLoader<TranslationCodec, Translations>>();
    app.init_resource::<CurrentTranslation>();
}

#[resource_kind(path = "images/ui", asset_kind = Image, file_type = ResourceFileType::Image)]
pub struct UiSpriteResource;

#[resource_kind(path = "text", asset_kind = (), file_type = ResourceFileType::None)]
pub struct UiTextResource;

#[resource_kind(path = "translations", asset_kind = Translations)]
pub struct TranslationResource;

pub trait Translator<T: ResourceKind> {
    fn translate(&self, loc: &ResourceLocation<T>) -> Option<&str>;
}

#[derive(Asset, Clone, TypePath)]
pub struct Translations {
    ui: TranslationList<UiTextResource>,
}
impl Translator<UiTextResource> for Translations {
    fn translate(&self, loc: &ResourceLocation<UiTextResource>) -> Option<&str> {
        self.ui.translate(loc)
    }
}

impl From<TranslationCodec> for Translations {
    fn from(codec: TranslationCodec) -> Self {
        Translations { ui: codec.ui }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct CurrentTranslation(pub ResourceLocation<TranslationResource>);
impl Default for CurrentTranslation {
    fn default() -> Self {
        CurrentTranslation("en_us".parse().unwrap())
    }
}
