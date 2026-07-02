use crate::theme::palette::BUTTON_TEXT;
use crate::theme::widgets::text::MEDIUM_FONT_SIZE;
use crate::theme::widgets::{button, UiBackgroundStyle};
use crate::theme::widgets;
use bevy::prelude::*;
use common::marker;

marker!(MenuBar);
marker!(MenuBarButtons);

const MENU_BUTTON_PER_CHAR_WIDTH: usize = 14;
const MENU_BUTTON_PADDING: usize = 12;
const MENU_BUTTON_HEIGHT: usize = 48;

const MENU_PADDING_VERTICAL: usize = 14;
const MENU_PADDING_HORIZONTAL: usize = 22;

pub(super) const MENU_BAR_TOTAL_HEIGHT: usize = MENU_BUTTON_HEIGHT + MENU_PADDING_VERTICAL * 2;

pub(super) fn spawn_menu_bar() -> impl Scene {
    bsn! [
        #MenuBar
        MenuBar
        widgets::ui_root()
        Node {
            flex_direction: FlexDirection::Row,

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
        }
        widgets::ui_background(UiBackgroundStyle::Main)
        Children [
            #MenuBarButtons
            MenuBarButtons
            widgets::ui_root()
            Node {
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
            }
            Children [
                button::with_text_ext(
                    "File",
                    MEDIUM_FONT_SIZE,
                    BUTTON_TEXT,
                    px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
                    percent(100),
                    file_button_clicked,
                ),
                button::with_text_ext(
                    "Edit",
                    MEDIUM_FONT_SIZE,
                    BUTTON_TEXT,
                    px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
                    percent(100),
                    edit_button_clicked,
                ),
                button::with_text_ext(
                    "View",
                    MEDIUM_FONT_SIZE,
                    BUTTON_TEXT,
                    px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
                    percent(100),
                    view_button_clicked,
                ),
                button::with_text_ext(
                    "Tools",
                    MEDIUM_FONT_SIZE,
                    BUTTON_TEXT,
                    px(MENU_BUTTON_PER_CHAR_WIDTH * 5 + MENU_BUTTON_PADDING),
                    percent(100),
                    tools_button_clicked,
                ),
                button::with_text_ext(
                    "Window",
                    MEDIUM_FONT_SIZE,
                    BUTTON_TEXT,
                    px(MENU_BUTTON_PER_CHAR_WIDTH * 6 + MENU_BUTTON_PADDING),
                    percent(100),
                    window_button_clicked,
                ),
            ]
        ]
    ]
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