//! Helper functions for creating common widgets.

use std::borrow::Cow;

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
};
use crate::asset_tracking::LoadResource;
use crate::theme::{interaction::InteractionPalette, palette::*};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<ButtonAssets>();
}

#[derive(Component, Debug)]
pub struct ButtonRoot;

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
pub fn header(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font_size(40.0),
        TextColor(HEADER_TEXT),
    )
}

/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::from_font_size(24.0),
        TextColor(LABEL_TEXT),
    )
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(
    button_assets: &ButtonAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    text: impl Into<String>,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        button_assets,
        texture_atlas_layouts,
        text,
        action,
        (Node {
            width: px(380),
            height: px(80),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },),
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(
    button_assets: &ButtonAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    text: impl Into<String>,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        button_assets,
        texture_atlas_layouts,
        text,
        action,
        Node {
            width: px(30),
            height: px(30),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base<E, B, M, I>(
    button_assets: &ButtonAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    text: impl Into<String>,
    action: I,
    button_bundle: impl Bundle,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let image = button_assets.button_sprite.clone();

    let action = IntoObserverSystem::into_system(action);

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 8, None, None);
    let layout = texture_atlas_layouts.add(layout);

    (
        Name::new("Button"),
        ButtonRoot,
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    ImageNode {
                        image,
                        image_mode: NodeImageMode::Sliced(ButtonSlicer::default().0),
                        texture_atlas: Some(TextureAtlas {
                            layout,
                            index: 0,
                        }),
                        ..default()
                    },
                    InteractionPalette {
                        none: 0,
                        hovered: 1,
                        pressed: 2,
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font_size(40.0),
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}

#[derive(Component)]
struct ButtonSlicer(TextureSlicer);

impl Default for ButtonSlicer {
    fn default() -> Self {
        Self(TextureSlicer {
            border: BorderRect::all(4.0), // Adjust based on your atlas design
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 16.0,
        })
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ButtonAssets {
    pub button_sprite: Handle<Image>,
}

impl FromWorld for ButtonAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            button_sprite: assets.load("images/ui/buttons.png"),
        }
    }
}
