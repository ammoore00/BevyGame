use crate::theme::palette::{HEADER_TEXT, LABEL_TEXT};
use bevy::prelude::*;

pub const TINY_FONT_SIZE: FontSize = FontSize::Px(16.0);
pub const SMALL_FONT_SIZE: FontSize = FontSize::Px(20.0);
pub const MEDIUM_FONT_SIZE: FontSize = FontSize::Px(24.0);
pub const LARGE_FONT_SIZE: FontSize = FontSize::Px(40.0);

pub fn text(text: impl Into<String>, size: impl Into<FontSize>, color: Color) -> impl Scene {
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
