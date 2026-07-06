use crate::button::style::ButtonStyle;
use crate::text;
use crate::theme::palette::BackgroundInteractionPalette;
use assets::resource::UiSpriteResource;
use bevy::ecs::template::OptionTemplate;
use bevy::image::TextureAtlasTemplate;
use bevy::prelude::*;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use data::loc::loc;

#[derive(SceneComponent, Debug, Clone, Copy, Eq, PartialEq, Default)]
#[scene(ButtonConfigProp)]
pub struct ButtonImpl;
impl ButtonImpl {
    fn scene(config: ButtonConfigProp) -> impl Scene {
        let (
            style,
            button_children,
            scene
        ) = match config.config.make_scene() {
            ButtonConfigScene::Styled {
                button_style,
                button_children,
                scene,
            } => (
                Box::new(bsn! [button_style]),
                button_children,
                scene,
            ),
            ButtonConfigScene::Background {
                background,
                button_children,
                scene
            } => (
                Box::new(bsn! [background]),
                button_children,
                scene,
            ),
        };

        bsn! [
            #Button
            ButtonImpl
            Button
            AutoDirectionalNavigation
            style
            Children [
                (
                    #ButtonChildren
                    button_children
                    Node {
                        justify_self: JustifySelf::Center,
                        padding: UiRect::all(Val::Px(5.0)),
                    }
                    Pickable::IGNORE
                )
            ]
            scene
        ]
    }
}

#[derive(Default)]
pub struct ButtonConfigProp {
    pub config: ButtonConfig
}

pub enum ButtonConfig {
    Styled {
        style: ButtonStyle,
        scene: Option<Box<dyn Scene>>,
    },
    Text {
        text: String,
        font_size: FontSize,
        color: Color,
        scene: Option<Box<dyn Scene>>,
    },
    TextInline {
        text: String,
        font_size: FontSize,
        color: Color,
        palette: BackgroundInteractionPalette,
        scene: Option<Box<dyn Scene>>,
    }
}
impl ButtonConfig {
    pub(crate) fn styled(style: ButtonStyle) -> Self {
        Self::Styled { style, scene: None }
    }

    pub(crate) fn text(text: String, font_size: FontSize, color: Color) -> Self {
        Self::Text { text, font_size, color, scene: None }
    }

    pub(crate) fn text_inline(text: String, font_size: FontSize, color: Color, palette: BackgroundInteractionPalette) -> Self {
        Self::TextInline { text, font_size, color, palette, scene: None }
    }

    pub(crate) fn with_scene(self, scene: impl Scene) -> Self {
        match self {
            Self::Styled { style, .. } =>
                Self::Styled { style, scene: Some(Box::new(scene)) },
            Self::Text { text, font_size, color, .. } =>
                Self::Text { text, font_size, color, scene: Some(Box::new(scene)) },
            Self::TextInline { text, font_size, color, palette, .. } =>
                Self::TextInline { text, font_size, color, palette, scene: Some(Box::new(scene)) },
        }
    }

    pub(crate) fn make_scene(self) -> ButtonConfigScene {
        match self {
            ButtonConfig::Styled { style, scene } => {
                let style = Box::new(bsn! [
                    {Self::make_scene_from_style(style)}
                ]);
                ButtonConfigScene::Styled {
                    button_style: style,
                    button_children: Box::new(bsn![]),
                    scene,
                }
            }
            ButtonConfig::Text {
                text,
                font_size,
                color,
                scene
            } => {
                let button_style = Box::new(bsn! [
                    {Self::make_scene_from_style(ButtonStyle::Default)}
                ]);
                let button_children = Box::new(bsn! [
                    {text::text(text, font_size, color)}
                ]);
                ButtonConfigScene::Styled {
                    button_style,
                    button_children,
                    scene,
                }
            }
            ButtonConfig::TextInline {
                text,
                font_size,
                color,
                palette,
                scene
            } => {
                let background = Box::new(bsn! [
                    BackgroundColor({palette.none})
                    BackgroundInteractionPalette {
                        none: {palette.none},
                        hovered: {palette.hovered},
                        pressed: {palette.pressed},
                    }
                ]);
                let button_children = Box::new(bsn! [
                    {text::text(text, font_size, color)}
                ]);
                ButtonConfigScene::Background {
                    background,
                    button_children,
                    scene,
                }
            }
        }
    }

    pub(crate) fn make_scene_from_style(style: ButtonStyle) -> impl Scene {
        bsn! [
            ImageNode {
                image: {loc::<UiSpriteResource>("buttons").unwrap()},
                image_mode: NodeImageMode::Sliced({style.make_slicer()}),
                texture_atlas: OptionTemplate::Some(TextureAtlasTemplate {
                    layout: asset_value(style.make_layout()),
                    index: {style.get_index()}
                }),
            }
            {style.make_palette_scene()}
        ]
    }
}
impl Default for ButtonConfig {
    fn default() -> Self {
        Self::styled(ButtonStyle::Default)
    }
}

enum ButtonConfigScene {
    Styled {
        button_style: Box<dyn Scene>,
        button_children: Box<dyn Scene>,
        scene: Option<Box<dyn Scene>>,
    },
    Background {
        background: Box<dyn Scene>,
        button_children: Box<dyn Scene>,
        scene: Option<Box<dyn Scene>>,
    }
}