//! Helper functions for creating common widgets.

use std::borrow::Cow;

use crate::asset_tracking::LoadResource;
use crate::menus::font::FontBuilder;
use crate::theme::{interaction::InteractionPalette, palette::*};
use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<UiAssets>();

    Assets::insert(
        &mut app.world_mut().resource_mut(),
        AssetId::default(),
        Font {
            data: include_bytes!("../../assets/base/fonts/bold_pixels.ttf")
                .to_vec()
                .into(),
        },
    )
    .expect("Failed to load font");

    app.add_systems(
        Update,
        (
            propagate_button_style_from_root,
            update_button_style
        ).chain()
    );
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

/// A simple header label. Bigger than [`label`].
pub fn header(
    text: impl Into<String>,
    font_builder: &FontBuilder,
) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        font_builder.with_size(40.0),
        TextColor(HEADER_TEXT),
    )
}

/// A simple text label.
pub fn label(
    text: impl Into<String>,
    font_builder: &FontBuilder,
) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        font_builder.with_size(24.0),
        TextColor(LABEL_TEXT),
    )
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(
    button_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    text: impl Into<String>,
    font_builder: &FontBuilder,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let font = font_builder.with_size(40.0);

    button_with_text(
        button_assets,
        texture_atlas_layouts,
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
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    text: impl Into<String>,
    font_builder: &FontBuilder,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let font = font_builder.with_size(24.0);

    button_with_text(
        ui_assets,
        texture_atlas_layouts,
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
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    text: impl Into<String>,
    width: Val,
    height: Val,
    font_size: f32,
    font_builder: &FontBuilder,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let font = font_builder.with_size(font_size);

    button_with_text(
        ui_assets,
        texture_atlas_layouts,
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
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
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
        ui_assets,
        texture_atlas_layouts,
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
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
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
        ui_assets,
        texture_atlas_layouts,
        action,
        button_bundle,
        button_children,
        style
    )
}

fn button_base<E, B, M, I>(
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
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
    let image = ui_assets.buttons.clone();

    let action = IntoObserverSystem::into_system(action);

    let layout = style.make_layout();
    let layout = texture_atlas_layouts.add(layout);

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
}

/// Detects button style applied to the button root
/// and propagates that component into the inner entity
fn propagate_button_style_from_root(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    root_query: Query<
        (
            Entity,
            &ButtonStyle,
            &Children
        ),
        (
            With<ButtonRoot>,
            Without<ButtonInner>,
            Changed<ButtonStyle>
        ),
    >,
    mut inner_query: Query<
        (
            &mut ButtonStyle,
            &Interaction,
            &mut ImageNode,
            &mut InteractionPalette,
        ),
        (
            With<ButtonInner>,
            Without<ButtonRoot>,
        ),
    >,
) {
    for (root_entity, style, children) in &root_query {
        for child in children {
            let Ok((mut inner_style, interaction, mut image_node, mut interaction_palette)) =
                inner_query.get_mut(*child)
            else {
                continue;
            };

            *inner_style = *style;

            apply_button_style(
                *style,
                *interaction,
                &mut texture_atlas_layouts,
                &mut image_node,
                &mut interaction_palette,
            );

            commands.entity(root_entity).remove::<ButtonStyle>();
            break;
        }
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
            &mut InteractionPalette,
        ),
        (With<ButtonInner>, Changed<ButtonStyle>),
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
    interaction_palette: &mut InteractionPalette,
) {
    let layout = texture_atlas_layouts.add(style.make_layout());
    let palette = style.make_interaction_palette();

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

    fn make_interaction_palette(&self) -> InteractionPalette {
        match self {
            ButtonStyle::Default => InteractionPalette {
                none: Self::idx(0, 0),
                hovered: Self::idx(0, 1),
                pressed: Self::idx(0, 2),
            },
            ButtonStyle::ArrowRight => InteractionPalette {
                none: Self::idx(1, 0),
                hovered: Self::idx(1, 1),
                pressed: Self::idx(1, 2),
            },
            ButtonStyle::ArrowDown => InteractionPalette {
                none: Self::idx(4, 0),
                hovered: Self::idx(4, 1),
                pressed: Self::idx(4, 2),
            },
            ButtonStyle::Back => InteractionPalette {
                none: Self::idx(7, 0),
                hovered: Self::idx(7, 1),
                pressed: Self::idx(7, 2),
            },
        }
    }

    fn idx(row: u32, col: u32) -> usize {
        (row * Self::COLS + col) as usize
    }
}

pub enum UiBackgroundStyle {
    Main,
    Panel,
}
impl UiBackgroundStyle {
    fn make_slicer(&self) -> TextureSlicer {
        match self {
            UiBackgroundStyle::Main => TextureSlicer {
                border: BorderRect::all(8.0),
                center_scale_mode: SliceScaleMode::Tile {
                    stretch_value: 1.0,
                },
                sides_scale_mode: SliceScaleMode::Tile {
                    stretch_value: 1.0,
                },
                max_corner_scale: 2.0,
            },
            UiBackgroundStyle::Panel => TextureSlicer {
                border: BorderRect::all(4.0),
                center_scale_mode: SliceScaleMode::Tile {
                    stretch_value: 1.0,
                },
                sides_scale_mode: SliceScaleMode::Tile {
                    stretch_value: 1.0,
                },
                max_corner_scale: 2.0,
            },
        }
    }

    fn make_layout(&self) -> TextureAtlasLayout {
        match self {
            UiBackgroundStyle::Main => TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 4, None, None),
            UiBackgroundStyle::Panel => TextureAtlasLayout::from_grid(UVec2::splat(24), 4, 4, Some(UVec2::splat(8)), Some(UVec2::splat(4))),
        }
    }

    fn get_index(&self) -> usize {
        match self {
            UiBackgroundStyle::Main => 0,
            UiBackgroundStyle::Panel => 1,
        }
    }
}

pub fn ui_background(
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    style: UiBackgroundStyle,
) -> impl Bundle {
    let image = ui_assets.background.clone();

    let layout = style.make_layout();
    let layout = texture_atlas_layouts.add(layout);

    let index = style.get_index();

    (
        ImageNode {
            image,
            image_mode: NodeImageMode::Sliced(style.make_slicer()),
            texture_atlas: Some(TextureAtlas { layout, index }),
            ..default()
        },
    )
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct UiAssets {
    pub buttons: Handle<Image>,
    pub background: Handle<Image>,
}

impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            buttons: assets.load("base/images/ui/buttons.png"),
            background: assets.load("base/images/ui/background.png"),
        }
    }
}
