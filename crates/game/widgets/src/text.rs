use crate::theme::palette::{HEADER_TEXT, LABEL_TEXT};
use bevy::prelude::*;
use common::{convert_world_to_screen_coords, Scale, WorldCoords};

pub const TINY_FONT_SIZE: FontSize = FontSize::Px(16.0);
pub const SMALL_FONT_SIZE: FontSize = FontSize::Px(20.0);
pub const MEDIUM_FONT_SIZE: FontSize = FontSize::Px(24.0);
pub const LARGE_FONT_SIZE: FontSize = FontSize::Px(40.0);

pub fn text(text: impl Into<String>, size: impl Into<FontSize>, color: Color) -> impl Scene {
    bsn! [
        #Text
        Text(text)
        text_formatting(size, color)
    ]
}

pub fn world_text(text: impl Into<String>, size: impl Into<FontSize>, color: Color, pos: WorldCoords, scale: Scale) -> impl Scene {
    bsn! [
        #Text2d
        Text2d(text)
        text_formatting(size, color)
        Transform {
            translation: {convert_world_to_screen_coords(scale, pos).0},
        }
    ]
}

fn text_formatting(size: impl Into<FontSize>, color: Color) -> impl Scene {
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
