use std::marker::PhantomData;
use bevy::ecs::query::QuerySingleError;
use bevy::ecs::relationship::Relationship;
use crate::theme::palette::HEADER_TEXT;
use crate::theme::widget::{styled_button, ButtonStyle, UiAssets};
use bevy::prelude::*;
use crate::menus::font::FontBuilder;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_animation_menu_contents,
        ).run_if(in_state(Screen::Editor))
    );
}

#[derive(Component, Debug, Clone, Default, Copy)]
struct FileBrowser;

pub(super) fn spawn_file_browser(
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    mut commands: Commands,
) -> Entity {
    let browser = commands.spawn((
        FileBrowser,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2.),

            width: percent(100),
            height: percent(100),

            ..Default::default()
        },
    )).id();

    let characters = collapsible_menu(
        ui_assets,
        texture_atlas_layouts,
        "Characters",
        TextFont::from_font_size(24.),
        commands.reborrow(),
    );
    commands.entity(characters).insert(CharacterMenu);
    commands.entity(browser).add_child(characters);

    let animations = collapsible_menu(
        ui_assets,
        texture_atlas_layouts,
        "Animations",
        TextFont::from_font_size(24.),
        commands.reborrow(),
    );
    commands.entity(animations).insert(AnimationMenu);
    commands.entity(browser).add_child(animations);

    let attacks = collapsible_menu(
        ui_assets,
        texture_atlas_layouts,
        "Attacks",
        TextFont::from_font_size(24.),
        commands.reborrow(),
    );
    commands.entity(attacks).insert(AttackMenu);
    commands.entity(browser).add_child(attacks);

    browser
}

#[derive(Component, Debug, Clone, Copy)]
struct Collapsed(bool);
impl Default for Collapsed { fn default() -> Self { Self(true) } }

fn collapsible_menu(
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    text: impl Into<String>,
    font: TextFont,
    mut commands: Commands,
) -> Entity {
    let menu = commands.spawn((
        Collapsed::default(),
        Node {
            flex_direction: FlexDirection::Column,

            width: percent(100),

            ..Default::default()
        },
    )).id();

    let menu_inner = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(8),

            width: percent(100),

            ..Default::default()
        },
    )).id();
    commands.entity(menu).add_child(menu_inner);

    let button = commands.spawn((
        styled_button(
            ui_assets,
            texture_atlas_layouts,
            2,
            |
                event: On<Pointer<Click>>,
                parent_query: Query<&ChildOf>,
                mut commands: Commands,
                mut menu_query: Query<&mut Collapsed>,
            | {
                let Ok(button_root) = parent_query.get(event.entity).map(ChildOf::get) else {
                    error!("Failed to get button root");
                    return;
                };

                let Ok(menu_inner) = parent_query.get(button_root).map(ChildOf::get) else {
                    error!("Failed to get menu inner");
                    return;
                };

                let Ok(menu) = parent_query.get(menu_inner).map(ChildOf::get) else {
                    error!("Failed to get menu root");
                    return;
                };

                if let Ok(mut collapsed) = menu_query.get_mut(menu) {
                    collapsed.0 = !collapsed.0;

                    let style = if collapsed.0 {
                        ButtonStyle::ArrowRight
                    } else {
                        ButtonStyle::ArrowDown
                    };

                    commands.entity(event.entity).insert(style);
                } else {
                    error!("Failed to get collapsed component for menu");
                }
            },
            ButtonStyle::ArrowRight,
        ),
    )).id();
    commands.entity(menu_inner).add_child(button);

    let text = commands.spawn((
        Text(text.into()),
        font,
        TextLayout {
            justify: Justify::Center,
            ..default()
        },
        TextColor(HEADER_TEXT),
    )).id();
    commands.entity(menu_inner).add_child(text);

    menu
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CharacterMenu;
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AnimationMenu;
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AttackMenu;

#[derive(Component, Debug)]
pub struct MenuContents<T: Component> {
    pub _phantom: PhantomData<T>,
}
impl<T: Component> MenuContents<T> {
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

const CONTENT_START_PADDING: f32 = 40.;
const CONTENT_PADDING: f32 = 4.;

fn update_animation_menu_contents(
    font_builder: FontBuilder,
    menu_query: Query<
        (
            Entity,
            &Collapsed
        ),
        With<AnimationMenu>
    >,
    contents_query: Query<
        Entity,
        With<MenuContents<AnimationMenu>>
    >,
    mut commands: Commands,
) {
    let Ok((menu, collapsed)) = menu_query.single() else {
        error!("Failed to get collapsed component for animation menu");
        return;
    };

    match contents_query.single() {
        Ok(contents) => {
            if collapsed.0 {
                commands.entity(contents).despawn()
            }
        }
        Err(QuerySingleError::NoEntities(_)) => {
            if !collapsed.0 {
                let contents = commands.spawn((
                    MenuContents::<AnimationMenu>::new(),
                    Node {
                        width: percent(100),
                        padding: UiRect::px(
                            CONTENT_START_PADDING,
                            CONTENT_PADDING,
                            CONTENT_PADDING,
                            CONTENT_PADDING,
                        ),

                        ..Default::default()
                    },
                    children![
                        Text("Content".to_string()),
                        font_builder.with_size(24.),
                        Node {
                            ..Default::default()
                        },
                    ]
                )).id();
                commands.entity(menu).add_child(contents);
            }
        }
        Err(QuerySingleError::MultipleEntities(_)) => error!("Multiple entities found for animation menu contents"),
    }
}