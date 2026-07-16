use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;
use crate::button::scene::{ButtonConfig, ButtonImpl};
use crate::button::style::ButtonStyle;
use crate::text::LARGE_FONT_SIZE;
use crate::theme::palette::{BackgroundInteractionPalette, BUTTON_TEXT};

#[derive(Debug, Clone)]
pub struct ButtonWithTextOptions {
    pub font_size: FontSize,
    pub color: Color,

    pub width: Val,
    pub height: Val,

    pub justify_content: JustifyContent,
}
impl Default for ButtonWithTextOptions {
    fn default() -> Self {
        Self {
            font_size: LARGE_FONT_SIZE,
            color: BUTTON_TEXT,

            width: px(380),
            height: px(80),

            justify_content: JustifyContent::Center,
        }
    }
}

pub fn with_text<E, B, M, I>(
    text: impl Into<String>,
    action: I,
) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync,
{
    with_text_ext(text, ButtonWithTextOptions::default(), action)
}

pub fn with_text_ext<E, B, M, I>(
    text: impl Into<String>,
    options: ButtonWithTextOptions,
    action: I,
) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync,
{
    let config = ButtonConfig::text(text.into(), options.font_size, options.color)
        .with_scene(bsn! [
            Node {
                width: {options.width},
                height: {options.height},
                align_items: AlignItems::Center,
                justify_content: {options.justify_content},
            }
        ]);

    base(config, action)
}

pub fn with_text_inline<E, B, M, I>(
    text: impl Into<String>,
    options: ButtonWithTextOptions,
    palette: BackgroundInteractionPalette,
    action: I,
) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync,
{
    let config = ButtonConfig::text_inline(text.into(), options.font_size, options.color, palette)
        .with_scene(bsn! [
            Node {
                width: {options.width},
                height: {options.height},
                align_items: AlignItems::Center,
                justify_content: {options.justify_content},
            }
        ]);

    base(config, action)
}

pub fn with_style<E, B, M, I>(
    style: ButtonStyle,
    scale: usize,
    action: I,
) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync,
{
    const BASE_SCALE: usize = 16;
    let size = scale * BASE_SCALE;

    let config = ButtonConfig::styled(style)
        .with_scene(bsn! [
            Node {
                width: px(size),
                height: px(size),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
        ]);

    base(config, action)
}

fn base<E, B, M, I>(
    config: ButtonConfig,
    action: I,
) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync,
{
    bsn! [
        @ButtonImpl {
            @config
        }
        on(action)
    ]
}