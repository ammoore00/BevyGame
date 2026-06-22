//! Helper functions for creating common widgets.

use std::borrow::Cow;

use crate::asset_tracking::LoadResource;
use crate::theme::widgets::text::FontBuilder;
use crate::theme::{interaction::InteractionPalette, palette::*};
use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
};
use bevy::ecs::system::SystemParam;
use crate::theme::widgets::button::ButtonStyle;
use crate::theme::widgets::text::{LARGE_FONT_SIZE, MEDIUM_FONT_SIZE};
use crate::theme::widgets::{button, UiAssets};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<UiAssets>();

    Assets::insert(
        &mut app.world_mut().resource_mut(),
        AssetId::default(),
        Font {
            data: include_bytes!("../../assets/base/fonts/bold_pixels.ttf")
                .to_vec()
                .into(),
            alias: "bold_pixels".to_string(),
        },
    )
    .expect("Failed to load font");
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ButtonRoot;

#[derive(Component, Debug, Clone, Copy, Default)]
struct ButtonInner;

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// A root UI node that fills the window and centers its content.
pub fn scrollable_ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            overflow: Overflow {
                x: OverflowAxis::Visible,
                y: OverflowAxis::Scroll,
            },
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label_old`].
pub fn header_old(
    text: impl Into<String>,
    font_builder: &FontBuilder,
) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        font_builder.with_size(LARGE_FONT_SIZE),
        TextColor(HEADER_TEXT),
        TextLayout {
            justify: Justify::Center,
            ..default()
        },
    )
}

/// A simple text label.
pub fn label_old(
    text: impl Into<String>,
    font_builder: &FontBuilder,
) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        font_builder.with_size(MEDIUM_FONT_SIZE),
        TextColor(LABEL_TEXT),
        TextLayout {
            justify: Justify::Center,
            ..default()
        },
    )
}

/// Text with the specified font size and color
pub fn text_old(
    text: impl Into<String>,
    font: TextFont,
    color: Color,
) -> TextBundle {
    TextBundle {
        text: Text(text.into()),
        font,
        color: TextColor(color),
        layout: TextLayout {
            justify: Justify::Center,
            ..default()
        },
    }
}

#[derive(Bundle)]
pub struct TextBundle {
    text: Text,
    font: TextFont,
    color: TextColor,
    layout: TextLayout,
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(
    ui_resources: &mut UiResources,
    text: impl Into<String>,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let font = ui_resources.font_builder.with_size(LARGE_FONT_SIZE);

    button_with_text(
        ui_resources,
        text,
        font,
        action,
        (Node {
            width: px(380),
            height: px(80),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },),
        ButtonStyle::Default,
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(
    ui_resources: &mut UiResources,
    text: impl Into<String>,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let font = ui_resources.font_builder.with_size(MEDIUM_FONT_SIZE);

    button_with_text(
        ui_resources,
        text,
        font,
        action,
        Node {
            width: px(30),
            height: px(30),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ButtonStyle::Default,
    )
}

/// A rounded button of the specified size with text and an action defined as an [`Observer`].
pub fn sized_button<E, B, M, I>(
    ui_resources: &mut UiResources,
    text: impl Into<String>,
    width: Val,
    height: Val,
    font_size: FontSize,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let font = ui_resources.font_builder.with_size(font_size);

    button_with_text(
        ui_resources,
        text,
        font,
        action,
        (
            Node {
                width,
                height,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ),
        ButtonStyle::Default,
    )
}

pub fn styled_button<E, B, M, I>(
    ui_resources: &mut UiResources,
    scale: usize,
    action: I,
    style: ButtonStyle,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    const BASE_SCALE: usize = 16;
    let size = scale * BASE_SCALE;

    button_base(
        ui_resources,
        action,
        Node {
            width: px(size),
            height: px(size),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        (),
        style,
    )
}

fn button_with_text<E, B, M, I>(
    ui_resources: &mut UiResources,
    text: impl Into<String>,
    font: TextFont,
    action: I,
    button_bundle: impl Bundle,
    style: ButtonStyle,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();

    let button_children = (
        Name::new("Button Text"),
        Text(text),
        font,
        TextColor(BUTTON_TEXT),
        TextLayout {
            justify: Justify::Center,
            ..default()
        },
    );

    button_base(
        ui_resources,
        action,
        button_bundle,
        button_children,
        style
    )
}

fn button_base<E, B, M, I>(
    ui_resources: &mut UiResources,
    action: I,
    button_bundle: impl Bundle,
    button_children: impl Bundle,
    style: ButtonStyle,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    /*
    let image = ui_resources.ui_assets.buttons.clone();

    let action = IntoObserverSystem::into_system(action);

    let layout = style.make_layout();
    let layout = ui_resources.texture_atlas_layouts.add(layout);

    let texture_slicer = style.make_slicer();

    let interaction_palette = style.make_interaction_palette();

    (
        Name::new("Button"),
        ButtonRoot,
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    ButtonInner,
                    style,
                    Button,
                    ImageNode {
                        image,
                        image_mode: NodeImageMode::Sliced(texture_slicer),
                        texture_atlas: Some(TextureAtlas { layout, index: 0 }),
                        ..default()
                    },
                    interaction_palette,
                    children![(
                        button_children,
                        Node {
                            justify_self: JustifySelf::Center,
                            padding: UiRect::all(Val::Px(5.0)),

                            ..default()
                        },
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
     */
}

#[derive(SystemParam)]
pub struct UiResources<'w> {
    pub ui_assets: Res<'w, UiAssets>,
    pub font_builder: FontBuilder<'w>,
    pub texture_atlas_layouts: ResMut<'w, Assets<TextureAtlasLayout>>,
}