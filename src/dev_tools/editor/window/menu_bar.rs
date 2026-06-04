use bevy::prelude::*;
use crate::menus::font::FontBuilder;
use crate::theme::widget;
use crate::theme::widget::{UiAssets, UiBackgroundStyle};

#[derive(Component, Debug, Clone, Default, Copy)]
struct MenuBarRoot;

#[derive(Component, Debug, Clone, Default, Copy)]
struct MenuBarButtonsRoot;


pub(super) const MENU_BUTTON_PER_CHAR_WIDTH: usize = 15;
pub(super) const MENU_BUTTON_PADDING: usize = 10;
pub(super) const MENU_BUTTON_HEIGHT: usize = 100;

pub(super) const MENU_PADDING_VERTICAL: usize = 14;
pub(super) const MENU_PADDING_HORIZONTAL: usize = 22;

pub(super) const MENU_BAR_BUTTON_HEIGHT: usize = 50;
pub(super) const MENU_BAR_TOTAL_HEIGHT: usize = MENU_BAR_BUTTON_HEIGHT + MENU_PADDING_VERTICAL * 2;

pub(super) fn spawn_menu_bar(
    ui_assets: &UiAssets,
    font_builder: &FontBuilder,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    mut commands: Commands
) -> Entity {
    let menu_bar = commands.spawn((
        MenuBarRoot,
        widget::ui_background(ui_assets, texture_atlas_layouts, UiBackgroundStyle::Main),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,

            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            right: px(0),
            height: px(MENU_BAR_TOTAL_HEIGHT),

            padding: UiRect::px(
                MENU_PADDING_HORIZONTAL as f32,
                MENU_PADDING_HORIZONTAL as f32,
                MENU_PADDING_VERTICAL as f32,
                MENU_PADDING_VERTICAL as f32,
            ),

            ..Default::default()
        },
    )).id();

    let menu_bar_buttons = commands.spawn((
        MenuBarButtonsRoot,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,

            width: percent(100),
            height: percent(100),

            ..Default::default()
        },
    )).id();
    commands.entity(menu_bar).add_child(menu_bar_buttons);

    let file_button = commands.spawn(widget::sized_button(
        ui_assets,
        texture_atlas_layouts,
        "File",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
        percent(MENU_BUTTON_HEIGHT),
        24.,
        font_builder,
        file_button_clicked,
    )).id();
    commands.entity(menu_bar_buttons).add_child(file_button);

    let edit_button = commands.spawn(widget::sized_button(
        ui_assets,
        texture_atlas_layouts,
        "Edit",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
        percent(MENU_BUTTON_HEIGHT),
        24.,
        font_builder,
        edit_button_clicked,
    )).id();
    commands.entity(menu_bar_buttons).add_child(edit_button);

    let view_button = commands.spawn(widget::sized_button(
        ui_assets,
        texture_atlas_layouts,
        "View",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
        percent(MENU_BUTTON_HEIGHT),
        24.,
        font_builder,
        view_button_clicked,
    )).id();
    commands.entity(menu_bar_buttons).add_child(view_button);

    let tools_button = commands.spawn(widget::sized_button(
        ui_assets,
        texture_atlas_layouts,
        "Tools",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 5 + MENU_BUTTON_PADDING),
        percent(MENU_BUTTON_HEIGHT),
        24.,
        font_builder,
        tools_button_clicked,
    )).id();
    commands.entity(menu_bar_buttons).add_child(tools_button);

    let window_button = commands.spawn(widget::sized_button(
        ui_assets,
        texture_atlas_layouts,
        "Window",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 6 + MENU_BUTTON_PADDING),
        percent(MENU_BUTTON_HEIGHT),
        24.,
        font_builder,
        window_button_clicked,
    )).id();
    commands.entity(menu_bar_buttons).add_child(window_button);

    menu_bar
}

fn file_button_clicked(
    _: On<Pointer<Click>>,
) {}

fn edit_button_clicked(
    _: On<Pointer<Click>>,
) {}

fn view_button_clicked(
    _: On<Pointer<Click>>,
) {}

fn tools_button_clicked(
    _: On<Pointer<Click>>,
) {}

fn window_button_clicked(
    _: On<Pointer<Click>>,
) {}