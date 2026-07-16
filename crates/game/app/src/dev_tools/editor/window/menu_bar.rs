use bevy::prelude::*;
use common::marker;
use widgets::background::UiBackgroundStyle;
use widgets::button;
use widgets::button::ButtonWithTextOptions;
use widgets::text::MEDIUM_FONT_SIZE;

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
        widgets::background::ui_root()
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
        widgets::background::ui_background(UiBackgroundStyle::Main)
        Children [
            #MenuBarButtons
            MenuBarButtons
            widgets::background::ui_root()
            Node {
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
            }
            Children [
                button::with_text_ext(
                    "File",
                    ButtonWithTextOptions {
                        font_size: MEDIUM_FONT_SIZE,
                        width: px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
                        height: percent(100.0),
                        ..default()    
                    },
                    file_button_clicked,
                ),
                button::with_text_ext(
                    "Edit",
                    ButtonWithTextOptions {
                        font_size: MEDIUM_FONT_SIZE,
                        width: px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
                        height: percent(100.0),
                        ..default()    
                    },
                    edit_button_clicked,
                ),
                button::with_text_ext(
                    "View",
                    ButtonWithTextOptions {
                        font_size: MEDIUM_FONT_SIZE,
                        width: px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
                        height: percent(100.0),
                        ..default()    
                    },
                    view_button_clicked,
                ),
                button::with_text_ext(
                    "Tools",
                    ButtonWithTextOptions {
                        font_size: MEDIUM_FONT_SIZE,
                        width: px(MENU_BUTTON_PER_CHAR_WIDTH * 5 + MENU_BUTTON_PADDING),
                        height: percent(100.0),
                        ..default()    
                    },
                    tools_button_clicked,
                ),
                button::with_text_ext(
                    "Window",
                    ButtonWithTextOptions {
                        font_size: MEDIUM_FONT_SIZE,
                        width: px(MENU_BUTTON_PER_CHAR_WIDTH * 6 + MENU_BUTTON_PADDING),
                        height: percent(100.0),
                        ..default()    
                    },
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