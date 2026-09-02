use crate::commands::parser::{CommandRegistry, parse_command};
use crate::window;
use bevy::input::common_conditions::input_just_pressed;
use bevy::input_focus::tab_navigation::{TabGroup, TabIndex};
use bevy::input_focus::{AutoFocus, InputFocus};
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use common::{GameState, Pause, marker};
use widgets::text::{TINY_FONT_SIZE, text};
use widgets::theme::palette::{ERROR_TEXT, PRIMARY_TEXT, SEPIA_2};

pub(super) fn plugin(app: &mut App) {
    app.init_state::<CommandsWindowOpen>();

    app.add_systems(
        Update,
        (
            set_commands_window_open.run_if(in_state(CommandsWindowOpen(false))),
            set_commands_window_closed.run_if(in_state(CommandsWindowOpen(true))),
        )
            .run_if(input_just_pressed(KeyCode::Backquote))
            // Don't just use GameplaySystems because we want to be able to use commands
            // even if main gameplay systems are suspended (GameplaySystems is not controlled
            // by Pause state, but might be separately suspended)
            .run_if(in_state(GameState::Gameplay)),
    );

    app.add_systems(
        OnEnter(CommandsWindowOpen(true)),
        spawn_command_window.spawn(),
    );

    app.add_systems(
        Update,
        command_submission.run_if(in_state(CommandsWindowOpen(true))),
    );

    app.add_observer(on_add_text);
}

marker!(CommandsWindow);

fn spawn_command_window() -> impl Scene {
    bsn! [
        #CommandWindow
        CommandsWindow
        window()
        Node {
            top: percent(70),
            height: percent(30),
            position_type: PositionType::Relative,
        }
        DespawnOnExit<CommandsWindowOpen>(CommandsWindowOpen(true))
        DespawnOnExit<GameState>(GameState::Gameplay)
        Children [(
            Node {
                height: percent(100),
                width: percent(100),

                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexEnd,
            }
            AutoFocus
            TabGroup::new(0)
            Children [
                (command_output()),
                (command_input()),
            ]
        )]
    ]
}

fn set_commands_window_open(
    mut state: ResMut<NextState<CommandsWindowOpen>>,
    mut paused: ResMut<NextState<Pause>>,
    paused_previous: Res<State<Pause>>,
) {
    state.set(CommandsWindowOpen(true));
    // Save the previous pause state to be restored later
    paused.set(Pause::ForcePaused(Box::new(paused_previous.clone())));
}

fn set_commands_window_closed(
    mut state: ResMut<NextState<CommandsWindowOpen>>,
    mut paused: ResMut<NextState<Pause>>,
    paused_previous: Res<State<Pause>>,
) {
    state.set(CommandsWindowOpen(false));

    // If the previous pause state was ForcePaused, restore the state before the force pause
    if let Pause::ForcePaused(prev) = paused_previous.clone() {
        paused.set(*prev);
    }
}

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandsWindowOpen(pub bool);

marker!(CommandInput);
marker!(CommandOutput);

fn command_input() -> impl Scene {
    bsn! [
        #CommandInput
        CommandInput
        Node {
            border: {px(2).all()},
            padding: {px(5).horizontal()}
        }
        EditableText {
            allow_newlines: false,
        }
        BorderColor::from(SEPIA_2)
        TextCursorStyle
        TabIndex(0)
        TextColor(PRIMARY_TEXT)
        TextLayout {
            justify: Justify::Left
        }
        TextFont {
            font_size: TINY_FONT_SIZE
        }
    ]
}

fn command_output() -> impl Scene {
    bsn! [
        #CommandOutput
        CommandOutput
        Node {
            border: {px(2).all()},
            padding: {px(5).horizontal()},

            flex_direction: FlexDirection::Column,

            align_items: AlignItems::FlexStart,
        }
    ]
}

fn command_submission(world: &mut World) {
    let keyboard_input = world.resource::<ButtonInput<KeyCode>>();
    let just_pressed_enter = keyboard_input.just_pressed(KeyCode::Enter);

    if !just_pressed_enter {
        return;
    }

    let input_focus = world.resource::<InputFocus>();
    let focused_entity = input_focus.get();

    let Some(focused_entity) = focused_entity else {
        return;
    };

    // Extract input text if the focused entity has EditableText
    let mut text_query = world.query_filtered::<&mut EditableText, With<CommandInput>>();
    let Ok(mut text_input) = text_query.get_mut(world, focused_entity) else {
        return;
    };

    let text_val = text_input.value().to_string();
    text_input.clear();

    // Parse command using registry
    let mut command_registry = world.resource_mut::<CommandRegistry>();
    let result = parse_command(&mut text_val.as_str(), &mut command_registry);

    let (output, color) = match result {
        Ok(command) => (command.invoke(world), PRIMARY_TEXT),
        Err(err) => (format!("Error: {}", err), ERROR_TEXT),
    };

    // Query for text output target entity
    let text_output_entity = world
        .query_filtered::<Entity, With<CommandOutput>>()
        .single(world)
        .unwrap();

    // Spawn and attach output text scene directly using World
    let text_entity = world
        .spawn_scene(text(output, TINY_FONT_SIZE, color))
        .unwrap()
        .id();

    world.entity_mut(text_output_entity).add_child(text_entity);
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct AddTextEvent(pub String);

fn on_add_text(
    event: On<AddTextEvent>,
    mut text_input: Single<&mut EditableText, With<CommandInput>>
) {
    let existing = text_input.value().to_string();
    let new_text = existing + &event.0;
    text_input.editor.set_text(new_text.as_str());
}