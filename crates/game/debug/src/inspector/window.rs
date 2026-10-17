use crate::inspector::{InspectorRegistry, InspectorWindowOpen};
use crate::{DebugPickable, window};
use bevy::asset::uuid::Uuid;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::scene::Ready;
use common::marker;
use widgets::text::{MEDIUM_FONT_SIZE, TINY_FONT_SIZE, TextContent, TextFormatting};
use widgets::theme::palette::PRIMARY_TEXT;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<InspectorSelection>();

    app.add_systems(
        Update,
        (
            update_inspector_data.run_if(in_state(InspectorWindowOpen(true))),
            (
                set_inspector_window_open.run_if(in_state(InspectorWindowOpen(false))),
                set_inspector_window_closed.run_if(in_state(InspectorWindowOpen(true))),
            )
                .run_if(input_just_pressed(KeyCode::F2)),
        ),
    );
    app.add_systems(OnEnter(InspectorWindowOpen(true)), inspector_window.spawn());

    app.add_observer(on_picked.run_if(in_state(InspectorWindowOpen(true))));
    app.add_observer(on_inspector_ready);
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

const SECTION_GAP: u32 = 20;
const ITEM_GAP: u32 = 8;

fn inspector() -> impl Scene {
    bsn! {
        @widgets::background::ui_root()
        Node {
            position_type: PositionType::Relative,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            justify_self: JustifySelf::Start,
            row_gap: px(SECTION_GAP),
        }
        Children [
            @header()
            --
            @data()
        ]
    }
}

marker!(InspectorSelectionDisplay);

fn header() -> impl Scene {
    bsn! {
        #InspectorHeader
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            justify_self: JustifySelf::Start,
            row_gap: px(ITEM_GAP),
        }
        Children [
            Node
            @widgets::text::text(
                "Inspector",
                TextFormatting {
                    font_size: MEDIUM_FONT_SIZE,
                    color: PRIMARY_TEXT,
                }
            )
            TextLayout {
                justify: Justify::Start
            }
            --
            #InspectorSelectionDisplay
            InspectorSelectionDisplay
            Node
            @widgets::text::text(
                "Click on an entity to inspect it!",
                TextFormatting {
                    font_size: TINY_FONT_SIZE,
                    color: PRIMARY_TEXT,
                }
            )
            TextLayout {
                justify: Justify::Start
            }
        ]
    }
}

/// If there is an entity selected already, set the inspector text to show it
fn on_inspector_ready(
    event: On<Ready>,
    inspector: Single<(Entity, &mut Text), With<InspectorSelectionDisplay>>,
    inspector_selection: Res<InspectorSelection>,
) {
    let (inspector_entity, mut inspector_text) = inspector.into_inner();

    if inspector_entity != event.entity {
        return;
    }

    let Some((_, debug_id)) = inspector_selection.0 else {
        return;
    };

    inspector_text.0 = format!("Inspecting: {:?}", debug_id);
}

#[derive(Resource, Debug, Clone, Default)]
struct InspectorSelection(Option<(Entity, Uuid)>);

/// Update the inspected entity
// TODO: Make this only happen when a button in the inspector is pressed so that normal gameplay won't interfere
fn on_picked(
    event: On<PointerPress>,
    pickable_query: Query<(Entity, &DebugPickable)>,
    inspector: Single<&mut Text, With<InspectorSelectionDisplay>>,
    mut inspector_selection: ResMut<InspectorSelection>,
) {
    if let Ok((entity, debug_id)) = pickable_query.get(event.entity) {
        let mut selection_text = inspector.into_inner();

        inspector_selection.0 = Some((entity, debug_id.0));
        selection_text.0 = format!("Inspecting: {:?}", debug_id.0);
    }
}

marker!(InspectorData);
marker!(InspectorDataEntry);

fn inspector_data_entry(text: impl Into<TextContent>) -> impl Scene {
    bsn! {
        InspectorDataEntry
        Node
        @widgets::text::text(
            text,
            TextFormatting {
                font_size: TINY_FONT_SIZE,
                color: PRIMARY_TEXT,
            }
        )
        TextLayout {
            justify: Justify::Start
        }
    }
}

fn data() -> impl Scene {
    bsn! {
        #InspectorData
        InspectorData
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            justify_self: JustifySelf::Start,
            row_gap: px(ITEM_GAP),
        }
    }
}

fn update_inspector_data(world: &mut World) {
    let Some((inspected_entity, _)) = world.resource::<InspectorSelection>().0 else {
        return;
    };

    let inspector_registry = world.resource::<InspectorRegistry>().clone();
    let inspector_output = match inspector_registry.get_matching(inspected_entity, &world.into()) {
        Ok(data) => (data.inspector_fn)(inspected_entity, &world.into()).into_data(),
        Err(err) => {
            let msg = format!("Failed to inspect entity: {err}");
            vec![msg]
        }
    };

    let mut inspector_display = world.query_filtered::<Entity, With<InspectorData>>();
    let inspector_display = match inspector_display.single(world) {
        Ok(display) => display,
        Err(err) => {
            error!("Failed to get inspector display: {err}");
            return;
        }
    };

    // Despawn existing entries
    let mut data_entries = world.query_filtered::<Entity, With<InspectorDataEntry>>();
    let existing = data_entries.query(world).iter().collect::<Vec<_>>();
    existing.iter().for_each(|entry| {
        world.despawn(*entry);
    });

    // Add new entries
    for entry in inspector_output {
        let entry = world.spawn_scene(inspector_data_entry(entry)).unwrap().id();
        world.entity_mut(inspector_display).add_child(entry);
    }
}
