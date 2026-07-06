use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;
use crate::button::scene::{ButtonConfig, ButtonImpl};
use crate::button::style::ButtonStyle;
use crate::text::LARGE_FONT_SIZE;
use crate::theme::palette::{BackgroundInteractionPalette, BUTTON_TEXT};

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
    with_text_ext(text, LARGE_FONT_SIZE, BUTTON_TEXT, px(380), px(80), action)
}

pub fn with_text_ext<E, B, M, I>(
    text: impl Into<String>,
    font_size: impl Into<FontSize>,
    text_color: Color,
    width: Val,
    height: Val,
    action: I,
) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync,
{
    let config = ButtonConfig::text(text.into(), font_size.into(), text_color)
        .with_scene(bsn! [
            Node {
                width,
                height,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
        ]);

    base(config, action)
}

pub fn with_text_inline<E, B, M, I>(
    text: impl Into<String>,
    font_size: impl Into<FontSize>,
    text_color: Color,
    width: Val,
    height: Val,
    palette: BackgroundInteractionPalette,
    action: I,
) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync,
{
    let config = ButtonConfig::text_inline(text.into(), font_size.into(), text_color, palette)
        .with_scene(bsn! [
            Node {
                width,
                height,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
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