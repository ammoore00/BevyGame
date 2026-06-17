use crate::data::loader::LoaderJobManager;
use crate::data::registry::ResourceRegistry;
use crate::data::{ResourceFileType, ResourceLocation, ResourceKind};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<FontResource>();

    app.insert_resource(DefaultFont("bold_pixels".parse().expect("Failed to parse default font")));
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct FontResource;
impl ResourceKind for FontResource {
    type AssetKind = Font;
    const ROOT_DIR: &'static str = "fonts";
    const FILE_TYPE: ResourceFileType = ResourceFileType::Font;
}

#[derive(Debug, Clone, Resource)]
struct DefaultFont(ResourceLocation<FontResource>);

#[derive(SystemParam)]
pub struct FontBuilder<'w> {
    default_font: Res<'w, DefaultFont>,
    font_registry: Res<'w, ResourceRegistry<FontResource>>
}
impl<'w> FontBuilder<'w> {
    /// Create text with the given size and the default font
    pub fn with_size(&self, font_size: f32) -> TextFont {
        self.with_font(font_size, self.default_font.0.clone())
            .expect(
                "Failed to load default font. This should never happen. Please report this bug.",
            )
    }

    /// Create text with the given size and font, using a resource location
    pub fn with_font(&self, font_size: f32, font: ResourceLocation<FontResource>) -> Option<TextFont> {
        font.get(self.font_registry.as_ref())
            .map(|font| TextFont {
                font: font.clone(),
                font_size,
                ..default()
            })
    }
}