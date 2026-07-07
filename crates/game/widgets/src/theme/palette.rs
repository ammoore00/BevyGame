use bevy::prelude::*;

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