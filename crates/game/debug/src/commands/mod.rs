mod parser;

use crate::window;
use bevy::input::common_conditions::input_just_pressed;
use bevy::input_focus::tab_navigation::{TabGroup, TabIndex};
use bevy::input_focus::{AutoFocus, InputFocus};
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use common::{GameState, Pause, marker};
use widgets::text::{TINY_FONT_SIZE, text};
use widgets::theme::palette::{BUTTON_TEXT, SEPIA_1};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(parser::plugin);

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
struct CommandsWindowOpen(bool);

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
        BorderColor::from(SEPIA_1)
        TextCursorStyle
        TabIndex(0)
        TextColor(BUTTON_TEXT)
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

            flex_direction: FlexDirection::ColumnReverse,
        }
    ]
}

fn command_submission(
    input_focus: Res<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_input: Query<&mut EditableText, With<CommandInput>>,
    text_output: Single<Entity, With<CommandOutput>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Enter)
        && let Some(focused_entity) = input_focus.get()
        && let Ok(mut text_input) = text_input.get_mut(focused_entity)
    {
        let text = commands
            .spawn_scene(text(
                format!("{:}", text_input.value()),
                TINY_FONT_SIZE,
                BUTTON_TEXT,
            ))
            .id();
        commands.entity(text_output.entity()).add_child(text);

        text_input.clear();
    }
}
