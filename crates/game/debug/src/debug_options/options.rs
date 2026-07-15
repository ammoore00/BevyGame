use bevy::prelude::*;
use common::dev_tools::*;
use widgets::text::{MEDIUM_FONT_SIZE, SMALL_FONT_SIZE, TINY_FONT_SIZE};
use widgets::theme::palette::BUTTON_TEXT;

pub(super) fn plugin(app: &mut App) {
    //app.insert_resource(GlobalUiDebugOptions { enabled: true, ..default() });
}

//------ Global Debug ------//

pub(super) fn global_debug() -> impl Scene {
    bsn! [
        #GlobalDebug
        widgets::background::ui_root()
        Node {
            position_type: PositionType::Relative,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            row_gap: px(8)
        }
        Children [
            (
                widgets::text::text("Debug Options", MEDIUM_FONT_SIZE, BUTTON_TEXT)
                Node {
                    justify_self: JustifySelf::Start,
                }
            ),
            navigation(),
            physics(),
            ui(),
        ]
    ]
}

fn debug_category(display: &str) -> impl Scene {
    bsn! [
        DebugCategory
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
        }
        Children [
            (
                Node
                Children [
                    (
                        widgets::text::text(display, SMALL_FONT_SIZE, BUTTON_TEXT)
                        Node {
                            justify_self: JustifySelf::Start,
                        }
                    ),
                ]
            )
        ]
    ]
}

macro_rules! debug_option_list {
    ($($option:expr),* $(,)?) => {
        bsn! [
            Children [
                Node {
                    padding: UiRect::left(px(24)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Start,
                }
                Children [
                    $(
                        {$option},
                    )*
                ]
            ]
        ]
    };
}

macro_rules! debug_option {
    ($option:ident, $display:literal) => {
        bsn! [
            DebugEntry
            $option
            Node
            Children [
                (
                    widgets::text::text($display, TINY_FONT_SIZE, BUTTON_TEXT)
                    Node {
                        justify_self: JustifySelf::Start,
                    }
                ),
            ]
        ]
    };
}

// Navigation

fn navigation() -> impl Scene {
    bsn! [
        #NavigationDebug
        debug_category("Navigation")
        {debug_option_list!(
            debug_option!(NavMapNodes, "Render Navigation Nodes"),
            debug_option!(NavMapEdges, "Render Navigation Edges"),
        )}
    ]
}

#[derive(Component, Default, Clone, DebugOption, Reflect)]
#[reflect(Component)]
pub struct NavMapNodes(bool);

#[derive(Component, Default, Clone, DebugOption, Reflect)]
#[reflect(Component)]
pub struct NavMapEdges(bool);

// Physics

fn physics() -> impl Scene {
    bsn! [
        #PhysicsDebug
        debug_category("Physics")
        {debug_option_list!(
            debug_option!(CharacterCollision, "Render Character Collision"),
            debug_option!(TileCollision, "Render Tile Collision"),
        )}
    ]
}

#[derive(Component, Default, Clone, DebugOption, Reflect)]
#[reflect(Component)]
pub struct CharacterCollision(bool);

#[derive(Component, Default, Clone, DebugOption, Reflect)]
#[reflect(Component)]
pub struct TileCollision(bool);

// User Interface

fn ui() -> impl Scene {
    bsn! [
        #UiDebug
        debug_category("User Interface")
        {debug_option_list!(
            debug_option!(UiRender, "Render User Interface Debug"),
        )}
    ]
}

#[derive(Component, Default, Clone, DebugOption, Reflect)]
#[reflect(Component)]
pub struct UiRender(bool);

//------ Per-Entity Debug ------//

// Navigation

#[derive(Component, Default, Clone, DebugOption, Reflect)]
#[reflect(Component)]
pub struct NpcPath(bool);