use bevy::prelude::*;
use crate::theme::{widget_old, widgets};
use crate::theme::widget_old::UiResources;
use crate::theme::widgets::text::MEDIUM_FONT_SIZE;
use crate::theme::widgets::UiBackgroundStyle;

#[derive(Component, Debug, Clone, Default, Copy)]
struct MenuBarRoot;

#[derive(Component, Debug, Clone, Default, Copy)]
struct MenuBarButtonsRoot;

pub(super) const MENU_BUTTON_PER_CHAR_WIDTH: usize = 14;
pub(super) const MENU_BUTTON_PADDING: usize = 12;
pub(super) const MENU_BUTTON_HEIGHT: usize = 48;

pub(super) const MENU_PADDING_VERTICAL: usize = 14;
pub(super) const MENU_PADDING_HORIZONTAL: usize = 22;

pub(super) const MENU_BAR_TOTAL_HEIGHT: usize = MENU_BUTTON_HEIGHT + MENU_PADDING_VERTICAL * 2;

pub(super) fn spawn_menu_bar() -> impl Scene {

}

pub(super) fn spawn_menu_bar_old(
    ui_resources: &mut UiResources,
    mut commands: Commands
) -> Entity {
    let menu_bar = commands.spawn((
        MenuBarRoot,
        widgets::ui_background_old(ui_resources, UiBackgroundStyle::Main),
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

    let file_button = commands.spawn(widget_old::sized_button(
        ui_resources,
        "File",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
        percent(100),
        MEDIUM_FONT_SIZE,
        file_button_clicked,
    )).id();
    commands.entity(menu_bar_buttons).add_child(file_button);

    let edit_button = commands.spawn(widget_old::sized_button(
        ui_resources,
        "Edit",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
        percent(100),
        MEDIUM_FONT_SIZE,
        edit_button_clicked,
    )).id();
    commands.entity(menu_bar_buttons).add_child(edit_button);

    let view_button = commands.spawn(widget_old::sized_button(
        ui_resources,
        "View",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
        percent(100),
        MEDIUM_FONT_SIZE,
        view_button_clicked,
    )).id();
    commands.entity(menu_bar_buttons).add_child(view_button);

    let tools_button = commands.spawn(widget_old::sized_button(
        ui_resources,
        "Tools",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 5 + MENU_BUTTON_PADDING),
        percent(100),
        MEDIUM_FONT_SIZE,
        tools_button_clicked,
    )).id();
    commands.entity(menu_bar_buttons).add_child(tools_button);

    let window_button = commands.spawn(widget_old::sized_button(
        ui_resources,
        "Window",
        px(MENU_BUTTON_PER_CHAR_WIDTH * 6 + MENU_BUTTON_PADDING),
        percent(100),
        MEDIUM_FONT_SIZE,
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