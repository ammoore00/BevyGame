use crate::theme::palette::{HEADER_TEXT, LABEL_TEXT};
use assets::resource::{CurrentTranslation, TranslationResource, Translations, Translator};
use bevy::prelude::*;
use common::{Scale, WorldCoords, convert_world_to_screen_coords};
use data::loc::AnyResourceLocation;
use data::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, translate_text);
}

pub const TINY_FONT_SIZE: FontSize = FontSize::Px(16.0);
pub const SMALL_FONT_SIZE: FontSize = FontSize::Px(20.0);
pub const MEDIUM_FONT_SIZE: FontSize = FontSize::Px(24.0);
pub const LARGE_FONT_SIZE: FontSize = FontSize::Px(40.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextFormatting {
    pub font_size: FontSize,
    pub color: Color,
}
impl TextFormatting {
    fn as_scene(&self) -> impl Scene {
        bsn! [
            TextColor({self.color})
            TextLayout {
                justify: Justify::Center
            }
            TextFont {
                font_size: {self.font_size}
            }
        ]
    }
}

type TranslatorFn = for<'a> fn(&AnyResourceLocation, &'a Translations) -> Option<&'a str>;

fn translate_with<'a, T>(
    loc: &AnyResourceLocation,
    translations: &'a Translations,
) -> Option<&'a str>
where
    T: ResourceKind,
    Translations: Translator<T>,
{
    let typed_loc = ResourceLocation::<T>::from(loc.clone());
    translations.translate(&typed_loc)
}

pub enum TextContent {
    Raw(String),
    Localized {
        loc: AnyResourceLocation,
        translator: TranslatorFn,
    },
}
impl TextContent {
    /// Raw text that should not be localized
    pub fn raw(text: impl Into<String>) -> Self {
        Self::from(text)
    }

    /// A resource location pointing to text that should be localized
    pub fn localized<T>(loc: ResourceLocation<T>) -> Self
    where
        T: ResourceKind,
        Translations: Translator<T>,
    {
        Self::Localized {
            loc: AnyResourceLocation::from(loc),
            translator: translate_with::<T>,
        }
    }

    fn as_scene(&self) -> Box<dyn Scene> {
        match self {
            TextContent::Raw(text) => Box::new(bsn! [
                #Text
                Text(text)
            ]),
            TextContent::Localized { loc, translator } => Box::new(bsn! [
                #LocalizedText
                LocalizableText {
                    loc: {loc.clone()},
                    translator: {*translator},
                }
                Text
            ]),
        }
    }
}
impl<S: Into<String>> From<S> for TextContent {
    fn from(value: S) -> Self {
        Self::Raw(value.into())
    }
}

#[derive(Component, Clone)]
struct LocalizableText {
    loc: AnyResourceLocation,
    translator: TranslatorFn,
    last_language: Option<ResourceLocation<TranslationResource>>,
}
impl Default for LocalizableText {
    // This function should not be used but is required for use in scenes
    fn default() -> Self {
        Self {
            loc: "invalid".parse().unwrap(),
            translator: |_, _| None,
            last_language: None,
        }
    }
}

fn translate_text(
    text_query: Query<(&mut LocalizableText, &mut Text)>,
    current_translation: Res<CurrentTranslation>,
    translation_registry: SystemRegistry<TranslationResource>,
) {
    for (mut localizable_text, mut text) in text_query {
        if localizable_text.last_language.as_ref() != Some(&current_translation.0) {
            let translations = translation_registry.get_asset(&current_translation.0);

            let translated = translations
                .map(|translations| {
                    (localizable_text.translator)(&localizable_text.loc, translations)
                })
                .flatten()
                .map(|s| s.to_string())
                .unwrap_or(localizable_text.loc.to_string());

            text.0 = translated;
            localizable_text.last_language = Some(current_translation.0.clone());
        }
    }
}

pub fn text(text: impl Into<String>, size: impl Into<FontSize>, color: Color) -> impl Scene {
    bsn! [
        #Text
        Text(text)
        text_formatting(size, color)
    ]
}

pub fn world_text(
    text: impl Into<String>,
    size: impl Into<FontSize>,
    color: Color,
    pos: WorldCoords,
    scale: Scale,
) -> impl Scene {
    bsn! [
        #Text2d
        Text2d(text)
        text_formatting(size, color)
        Transform {
            translation: {convert_world_to_screen_coords(scale, pos).0},
        }
    ]
}

pub fn text_formatting(size: impl Into<FontSize>, color: Color) -> impl Scene {
    bsn! [
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
