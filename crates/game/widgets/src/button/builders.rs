use crate::button::scene::{ButtonConfig, ButtonImpl};
use crate::button::style::ButtonStyle;
use crate::text::LARGE_FONT_SIZE;
use crate::theme::palette::{BackgroundInteractionPalette, PRIMARY_TEXT};
use bevy::ecs::system::IntoObserverSystem;
use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::scene::{ResolveContext, ResolveSceneError, ResolvedScene};
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;

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
            color: PRIMARY_TEXT,

            width: px(380),
            height: px(80),

            justify_content: JustifyContent::Center,
        }
    }
}

pub struct ButtonScene {
    inner: Box<dyn Scene>,
}
impl ButtonScene {
    pub fn new(scene: impl Scene + 'static) -> Self {
        Self {
            inner: Box::new(scene),
        }
    }
    
    pub fn with_automatic_navigation(self) -> Self {
        Self {
            inner: Box::new(bsn! {
                @{self.inner}
                AutoDirectionalNavigation
            })
        }
    }
    
    pub fn with_automatic_navigation_focused(self) -> Self {
        Self {
            inner: Box::new(bsn! {
                @{self.inner}
                AutoDirectionalNavigation
                AutoFocus
            })
        }
    }
}
impl Scene for ButtonScene {
    fn resolve(self, context: &mut ResolveContext, scene: &mut ResolvedScene) -> Result<(), ResolveSceneError> {
        self.inner.resolve(context, scene)
    }
}
impl From<Box<dyn Scene>> for ButtonScene {
    fn from(inner: Box<dyn Scene>) -> Self {
        Self {
            inner
        }
    }
}

pub fn with_text<E, M, I>(text: impl Into<String>, action: I) -> ButtonScene
where
    E: EntityEvent,
    M: 'static,
    I: IntoObserverSystem<E, M> + Clone + Send + Sync,
{
    with_text_ext(text, ButtonWithTextOptions::default(), action)
}

pub fn with_text_ext<E, M, I>(
    text: impl Into<String>,
    options: ButtonWithTextOptions,
    action: I,
) -> ButtonScene
where
    E: EntityEvent,
    M: 'static,
    I: IntoObserverSystem<E, M> + Clone + Send + Sync,
{
    let config =
        ButtonConfig::text(text.into(), options.font_size, options.color).with_scene(bsn! {
            Node {
                width: { options.width },
                height: { options.height },
                align_items: AlignItems::Center,
                justify_content: { options.justify_content },
            }
        });

    base(config, action)
}

pub fn with_text_inline<E, M, I>(
    text: impl Into<String>,
    options: ButtonWithTextOptions,
    palette: BackgroundInteractionPalette,
    action: I,
) -> ButtonScene
where
    E: EntityEvent,
    M: 'static,
    I: IntoObserverSystem<E, M> + Clone + Send + Sync,
{
    let config = ButtonConfig::text_inline(text.into(), options.font_size, options.color, palette)
        .with_scene(bsn! {
            Node {
                width: { options.width },
                height: { options.height },
                align_items: AlignItems::Center,
                justify_content: { options.justify_content },
            }
        });

    base(config, action)
}

pub fn with_style<E, M, I>(style: ButtonStyle, scale: usize, action: I) -> ButtonScene
where
    E: EntityEvent,
    M: 'static,
    I: IntoObserverSystem<E, M> + Clone + Send + Sync,
{
    const BASE_SCALE: usize = 16;
    let size = scale * BASE_SCALE;

    let config = ButtonConfig::styled(style).with_scene(bsn! {
        Node {
            width: px(size),
            height: px(size),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
    });

    base(config, action)
}

fn base<E, M, I>(config: ButtonConfig, action: I) -> ButtonScene
where
    E: EntityEvent,
    M: 'static,
    I: IntoObserverSystem<E, M> + Clone + Send + Sync,
{
    ButtonScene::new(bsn! {
        @ButtonImpl {
            @config
        }
        on(action)
    })
}
