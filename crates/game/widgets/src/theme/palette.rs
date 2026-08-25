use bevy::prelude::*;

// Main palette colors using Zorn palette

pub const SEPIA_0: Color = color(0x211a17);
pub const SEPIA_1: Color = color(0x534030);
pub const SEPIA_2: Color = color(0x765d46);
pub const SEPIA_3: Color = color(0x987e5f);
pub const SEPIA_4: Color = color(0xb29a78);
pub const SEPIA_5: Color = color(0xcbb390);
pub const SEPIA_6: Color = color(0xddcdb0);
pub const SEPIA_7: Color = color(0xf1e9d5);

pub const RED_1: Color = color(0x40241b);
pub const RED_2: Color = color(0x68382e);
pub const RED_3: Color = color(0x9e5a47);
pub const RED_4: Color = color(0xaf8771);
pub const RED_5: Color = color(0xd1b59e);
pub const RED_6: Color = color(0xe6cfb7);

pub const GREEN_1: Color = color(0x2b2921);
pub const GREEN_2: Color = color(0x4b442e);
pub const GREEN_3: Color = color(0x695d3e);
pub const GREEN_4: Color = color(0x8d7c53);
pub const GREEN_5: Color = color(0xb59d6d);
pub const GREEN_6: Color = color(0xd3be8f);

pub const BLUE_1: Color = color(0x342f2e);
pub const BLUE_2: Color = color(0x5a5655);
pub const BLUE_3: Color = color(0x736d6c);
pub const BLUE_4: Color = color(0x8d8684);
pub const BLUE_5: Color = color(0xc0b8b5);
pub const BLUE_6: Color = color(0xe3e0dd);

/// Const reimplementation of `Color::srgb_u32`
///
/// There is nothing stopping that function from being const, it just isn't for some reason
const fn color(code: u32) -> Color {
    Color::srgb(
        ((code >> 16) & 0xff) as f32 / 255.,
        ((code >> 8) & 0xff) as f32 / 255.,
        (code & 0xff) as f32 / 255.,
    )
}

/// #ddd369
pub const LABEL_TEXT: Color = Color::srgb(0.867, 0.827, 0.412);

/// #fcfbcc
pub const HEADER_TEXT: Color = Color::srgb(0.988, 0.984, 0.800);

/// #ececec
pub const BUTTON_TEXT: Color = Color::srgb(0.925, 0.925, 0.925);
/// #4666bf
pub const BUTTON_BACKGROUND: Color = Color::srgb(0.275, 0.400, 0.750);
/// #6299d1
pub const BUTTON_HOVERED_BACKGROUND: Color = Color::srgb(0.384, 0.600, 0.820);
/// #3d4999
pub const BUTTON_PRESSED_BACKGROUND: Color = Color::srgb(0.239, 0.286, 0.600);

/// #6b5052
pub const TEXT_INPUT_BACKGROUND: Color = Color::srgb(0.420, 0.314, 0.322);

pub const TRANSPARENT_OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect, FromTemplate)]
#[reflect(Component)]
pub struct SpriteInteractionPalette {
    pub none: usize,
    pub hovered: usize,
    pub pressed: usize,
}

#[derive(Component, Debug, Reflect, FromTemplate)]
#[reflect(Component)]
pub struct BackgroundInteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}
