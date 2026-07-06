use crate::dev_tools::editor::window::browser::spawn_file_browser;
use crate::dev_tools::editor::window::editor_port::spawn_editor_port;
use crate::dev_tools::editor::window::menu_bar::{spawn_menu_bar, MENU_BAR_TOTAL_HEIGHT};
use crate::dev_tools::editor::window::properties::spawn_details_screen;
use crate::screens::Screen;
use bevy::prelude::*;
use common::marker;
use widgets::background::UiBackgroundStyle;

mod menu_bar;
mod browser;
mod editor_port;
pub(super) mod properties;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        browser::plugin,
        properties::plugin,
        editor_port::plugin,
    ));

    app.add_systems(OnEnter(Screen::Editor), spawn_editor.spawn());
}


marker!(EditorUiRoot);
marker!(EditorContentRoot);
marker!(EditorLeftPanel);
marker!(EditorCenterPanel);
marker!(EditorRightPanel);
marker!(EditorBottomPanel);

const CENTER_PANEL_HEIGHT: usize = 70;
const LOWER_PANEL_HEIGHT: usize = 100 - CENTER_PANEL_HEIGHT;

const LEFT_PANEL_WIDTH_TARGET: usize = 25;
const RIGHT_PANEL_WIDTH_TARGET: usize = 25;

const LEFT_PANEL_MAX_WIDTH: usize = 500;
const RIGHT_PANEL_MAX_WIDTH: usize = 800;

const PANEL_PADDING: usize = 14;

const BACKGROUND_BLEED: f32 = 10.0;

fn spawn_editor() -> impl Scene {
    bsn! {
        #EditorUiRoot
        EditorUiRoot
        widgets::background::ui_root()
        Node {
            row_gap: px(0),
        }
        Children [
            (
                #EditorContentRoot
                EditorContentRoot
                Node {
                    flex_direction: FlexDirection::Column,

                    position_type: PositionType::Absolute,
                    top: px(MENU_BAR_TOTAL_HEIGHT),
                    left: px(0),
                    right: px(0),
                    bottom: px(0),
                }
                Pickable::IGNORE
                Children [
                    (
                        #CenterPanelsRoot
                        Node {
                            width: percent(100),
                            height: percent(CENTER_PANEL_HEIGHT),
                            padding: UiRect::horizontal(px(2)),
                        }
                        Pickable::IGNORE
                        Children [
                            (
                                #LeftPanel
                                EditorLeftPanel
                                widgets::background::scrollable_ui_root()
                                Node {
                                    position_type: PositionType::Relative,
                                    width: percent(LEFT_PANEL_WIDTH_TARGET),
                                    max_width: px(LEFT_PANEL_MAX_WIDTH),
                                    padding: px(PANEL_PADDING),
                                }
                                Pickable::IGNORE
                                Children [
                                    (
                                        #LeftPanelBackground
                                        background()
                                    ),
                                    spawn_file_browser(),
                                ]
                            ),
                            (
                                #CenterPanel
                                EditorCenterPanel
                                widgets::background::ui_root()
                                Node {
                                    position_type: PositionType::Relative,
                                    flex_grow: 1.0,
                                }
                                Pickable::IGNORE
                                Children [
                                    spawn_editor_port(),
                                ]
                            ),
                            (
                                #RightPanel
                                EditorRightPanel
                                widgets::background::scrollable_ui_root()
                                Node {
                                    position_type: PositionType::Relative,
                                    width: percent(RIGHT_PANEL_WIDTH_TARGET),
                                    max_width: px(RIGHT_PANEL_MAX_WIDTH),
                                    padding: px(PANEL_PADDING),
                                }
                                Pickable::IGNORE
                                Children [
                                    (
                                        #RightPanelBackground
                                        background()
                                    ),
                                    spawn_details_screen()
                                ]
                            ),
                        ]
                    ),
                    (
                        #BottomPanel
                        EditorBottomPanel
                        widgets::background::ui_background(UiBackgroundStyle::Main)
                        Node {
                            width: percent(100),
                            height: percent(LOWER_PANEL_HEIGHT),
                        }
                        Pickable::IGNORE
                    ),
                ]
            ),
            // Menu Bar
            (
                spawn_menu_bar()
            ),
        ]
    }
}

fn background() -> impl Scene {
    bsn! [
        widgets::background::ui_background(UiBackgroundStyle::Panel)
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(-BACKGROUND_BLEED),
            bottom: px(-BACKGROUND_BLEED),
        }
        Pickable::IGNORE
    ]
}