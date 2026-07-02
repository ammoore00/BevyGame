use crate::theme::interaction::{BackgroundInteractionPalette, SpriteInteractionPalette};
use crate::theme::palette::BUTTON_TEXT;
use crate::theme::widgets::text::LARGE_FONT_SIZE;
use crate::theme::widgets::text;
use assets::resource::UiSpriteResource;
use bevy::ecs::system::IntoObserverSystem;
use bevy::ecs::template::OptionTemplate;
use bevy::image::TextureAtlasTemplate;
use bevy::prelude::*;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use data::prelude::loc;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_button_style
    );
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
    config: ButtonConfig
}

enum ButtonConfig {
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
    fn styled(style: ButtonStyle) -> Self {
        Self::Styled { style, scene: None }
    }

    fn text(text: String, font_size: FontSize, color: Color) -> Self {
        Self::Text { text, font_size, color, scene: None }
    }

    fn text_inline(text: String, font_size: FontSize, color: Color, palette: BackgroundInteractionPalette) -> Self {
        Self::TextInline { text, font_size, color, palette, scene: None }
    }

    fn with_scene(self, scene: impl Scene) -> Self {
        match self {
            Self::Styled { style, .. } =>
                Self::Styled { style, scene: Some(Box::new(scene)) },
            Self::Text { text, font_size, color, .. } =>
                Self::Text { text, font_size, color, scene: Some(Box::new(scene)) },
            Self::TextInline { text, font_size, color, palette, .. } =>
                Self::TextInline { text, font_size, color, palette, scene: Some(Box::new(scene)) },
        }
    }

    fn make_scene(self) -> ButtonConfigScene {
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

    fn make_scene_from_style(style: ButtonStyle) -> impl Scene {
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

#[derive(Component, Debug, Clone, Copy, Default)]
pub enum ButtonStyle {
    #[default]
    Default,
    ArrowRight,
    //ArrowLeft,
    //ArrowUp,
    ArrowDown,
    //Plus,
    //Minus,
    Back,
}

impl ButtonStyle {
    const ROWS: u32 = 8;
    const COLS: u32 = 8;

    fn make_slicer(&self) -> TextureSlicer {
        TextureSlicer {
            border: BorderRect::all(4.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 16.0,
        }
    }

    // TODO: Cache these layouts
    fn make_layout(&self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(UVec2::splat(16), Self::COLS, Self::ROWS, None, None)
    }

    fn get_indices(&self) -> (usize, usize, usize) {
        match self {
            ButtonStyle::Default => (
                Self::idx(0, 0),
                Self::idx(0, 1),
                Self::idx(0, 2)
            ),
            ButtonStyle::ArrowRight => (
                Self::idx(1, 0),
                Self::idx(1, 1),
                Self::idx(1, 2)
            ),
            ButtonStyle::ArrowDown => (
                Self::idx(4, 0),
                Self::idx(4, 1),
                Self::idx(4, 2)
            ),
            ButtonStyle::Back => (
                Self::idx(7, 0),
                Self::idx(7, 1),
                Self::idx(7, 2)
            ),
        }
    }

    fn make_palette_scene(self) -> impl Scene {
        let indices = self.get_indices();
        bsn! [
            SpriteInteractionPalette {
                none: {indices.0},
                hovered: {indices.1},
                pressed: {indices.2},
            }
        ]
    }

    fn get_palette(&self) -> SpriteInteractionPalette {
        let indices = self.get_indices();
        SpriteInteractionPalette {
            none: {indices.0},
            hovered: {indices.1},
            pressed: {indices.2},
        }
    }

    fn idx(row: u32, col: u32) -> usize {
        (row * Self::COLS + col) as usize
    }

    fn get_index(&self) -> usize {
        self.get_indices().0
    }
}

/// Detects changes to the button style and applies the appropriate visual state.
fn update_button_style(
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut button_query: Query<
        (
            &ButtonStyle,
            &Interaction,
            &mut ImageNode,
            &mut SpriteInteractionPalette,
        ),
        (
            With<ButtonImpl>,
            Changed<ButtonStyle>
        ),
    >,
) {
    for (style, interaction, mut image_node, mut interaction_palette) in &mut button_query {
        apply_button_style(
            *style,
            *interaction,
            &mut texture_atlas_layouts,
            &mut image_node,
            &mut interaction_palette,
        );
    }
}

fn apply_button_style(
    style: ButtonStyle,
    interaction: Interaction,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    image_node: &mut ImageNode,
    interaction_palette: &mut SpriteInteractionPalette,
) {
    let layout = texture_atlas_layouts.add(style.make_layout());
    let palette = style.get_palette();

    let index = match interaction {
        Interaction::None => palette.none,
        Interaction::Hovered => palette.hovered,
        Interaction::Pressed => palette.pressed,
    };

    image_node.image_mode = NodeImageMode::Sliced(style.make_slicer());
    image_node.texture_atlas = Some(TextureAtlas {
        layout,
        index,
    });

    *interaction_palette = palette;
}