use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use crate::data::registry::ResourceRegistry;
use crate::data::{ResourceFileType, ResourceKind, ResourceLocation};
use crate::data::loader::LoaderJobManager;
use crate::theme::palette::{HEADER_TEXT, LABEL_TEXT};

pub(crate) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<FontResource>();

    app.insert_resource(DefaultFont("bold_pixels".parse().expect("Failed to parse default font")));
}

pub const TINY_FONT_SIZE: FontSize = FontSize::Px(16.0);
pub const SMALL_FONT_SIZE: FontSize = FontSize::Px(20.0);
pub const MEDIUM_FONT_SIZE: FontSize = FontSize::Px(24.0);
pub const LARGE_FONT_SIZE: FontSize = FontSize::Px(40.0);

pub fn text(
    text: impl Into<String>,
    size: impl Into<FontSize>,
    color: Color,
) -> impl Scene {
    bsn! [
        #Text
        Text(text)
        TextColor(color)
        TextLayout {
            justify: Justify::Center
        }
        TextFont {
            font_size: size
        }
    ]
}

pub fn label(text_str: impl Into<String>) -> impl Scene {
    text(text_str, MEDIUM_FONT_SIZE, LABEL_TEXT)
}

pub fn header(text_str: impl Into<String>) -> impl Scene {
    text(text_str, LARGE_FONT_SIZE, HEADER_TEXT)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct FontResource;

impl ResourceKind for FontResource {
    type AssetKind = Font;
    const ROOT_DIR: &'static str = "fonts";
    const FILE_TYPE: ResourceFileType = ResourceFileType::Font;
}

#[derive(Debug, Clone, Resource)]
pub struct DefaultFont(ResourceLocation<FontResource>);

#[derive(SystemParam)]
pub struct FontBuilder<'w> {
    default_font: Res<'w, DefaultFont>,
    font_registry: Res<'w, ResourceRegistry<FontResource>>
}

impl<'w> FontBuilder<'w> {
    /// Create text with the given size and the default font
    pub fn with_size(&self, font_size: FontSize) -> TextFont {
        self.with_font(font_size, self.default_font.0.clone())
            .expect(
                "Failed to load default font. This should never happen. Please report this bug.",
            )
    }

    /// Create text with the given size and font, using a resource location
    pub fn with_font(&self, font_size: FontSize, font: ResourceLocation<FontResource>) -> Option<TextFont> {
        font.get(self.font_registry.as_ref())
            .map(|font| TextFont {
                font: FontSource::from(font),
                font_size,
                ..default()
            })
    }
}