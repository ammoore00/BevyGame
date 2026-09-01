use crate::loader::LoaderJobManager;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use data::loc::ResourceLocation;
use data::prelude::{ResourceFileType, ResourceRegistry};
use data::resource::resource_kind;

pub(super) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<FontResource>();

    Assets::insert(
        &mut app.world_mut().resource_mut(),
        AssetId::default(),
        Font {
            data: include_bytes!("../../../../../assets/base/fonts/bold_pixels.ttf")
                .to_vec()
                .into(),
            alias: "bold_pixels".to_string(),
        },
    )
    .expect("Failed to load font");

    app.insert_resource(DefaultFont(
        "bold_pixels".parse().expect("Failed to parse default font"),
    ));
}

#[resource_kind(path = "fonts", asset_kind = Font, file_type = ResourceFileType::Font)]
pub struct FontResource;

#[derive(Debug, Clone, Resource)]
struct DefaultFont(ResourceLocation<FontResource>);

#[derive(SystemParam)]
pub struct FontBuilder<'w> {
    default_font: Res<'w, DefaultFont>,
    font_registry: Res<'w, ResourceRegistry<FontResource>>,
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
    pub fn with_font(
        &self,
        font_size: FontSize,
        font: ResourceLocation<FontResource>,
    ) -> Option<TextFont> {
        font.get(self.font_registry.as_ref()).map(|font| TextFont {
            font: FontSource::from(font),
            font_size,
            ..default()
        })
    }
}
