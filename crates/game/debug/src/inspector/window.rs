use crate::inspector::InspectorWindowOpen;
use crate::{DebugPickable, window};
use bevy::asset::uuid::Uuid;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use common::marker;
use widgets::text::{MEDIUM_FONT_SIZE, TINY_FONT_SIZE, TextFormatting};
use widgets::theme::palette::PRIMARY_TEXT;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            set_inspector_window_open.run_if(in_state(InspectorWindowOpen(false))),
            set_inspector_window_closed.run_if(in_state(InspectorWindowOpen(true))),
        )
            .run_if(input_just_pressed(KeyCode::F2)),
    );
    app.add_systems(OnEnter(InspectorWindowOpen(true)), inspector_window.spawn());

    app.add_observer(on_picked);
}

fn set_inspector_window_open(mut state: ResMut<NextState<InspectorWindowOpen>>) {
    state.set(InspectorWindowOpen(true));
}

fn set_inspector_window_closed(mut state: ResMut<NextState<InspectorWindowOpen>>) {
    state.set(InspectorWindowOpen(false));
}

marker!(InspectorWindow);

fn inspector_window() -> impl Scene {
    bsn! {
        #InspectorWindow
        InspectorWindow
        @window()
        Node {
            position_type: PositionType::Absolute,
            right: px(0),
            top: px(0),
            width: percent(40),
            padding: UiRect::all(px(16)),
        }
        DespawnOnExit<InspectorWindowOpen>(InspectorWindowOpen(true))
        Children [
            @inspector()
        ]
    }
}

fn inspector() -> impl Scene {
    bsn! {
        @widgets::background::ui_root()
        Node {
            position_type: PositionType::Relative,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            justify_self: JustifySelf::Start,
            row_gap: px(8)
        }
        Children [
            Node {
                justify_self: JustifySelf::Start,
            }
            @widgets::text::text(
                "Inspector",
                TextFormatting {
                    font_size: MEDIUM_FONT_SIZE,
                    color: PRIMARY_TEXT,
                }
            )
            --
            #InspectorSelection
            InspectorSelection
            Node {
                justify_self: JustifySelf::Start,
            }
            @widgets::text::text(
                "Click on an entity to inspect it!",
                TextFormatting {
                    font_size: TINY_FONT_SIZE,
                    color: PRIMARY_TEXT,
                }
            )
        ]
    }
}

#[derive(Component, Debug, Clone, Default)]
struct InspectorSelection(Option<(Entity, Uuid)>);

fn on_picked(
    event: On<PointerPress>,
    pickable_query: Query<(Entity, &DebugPickable)>,
    inspector: Single<(&mut InspectorSelection, &mut Text)>,
) {
    if let Ok((entity, debug_id)) = pickable_query.get(event.entity) {
        let (mut inspector_selection, mut selection_text) = inspector.into_inner();

        inspector_selection.0 = Some((entity, debug_id.0));
        selection_text.0 = format!("Inspecting: {:?}", debug_id.0);
    }
}
