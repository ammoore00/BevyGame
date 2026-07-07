use crate::window;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use common::{marker, GameState};

pub(super) fn plugin(app: &mut App) {
    app.init_state::<CommandsWindowOpen>();

    app.add_systems(
        Update,
        (
            set_commands_window_open.run_if(in_state(CommandsWindowOpen(false))),
            set_commands_window_closed.run_if(in_state(CommandsWindowOpen(true))),
        )
            .run_if(input_just_pressed(KeyCode::Backquote))
            .run_if(in_state(GameState::Gameplay)),
    );

    app.add_systems(OnEnter(CommandsWindowOpen(true)), spawn_command_window.spawn());
}

marker!(CommandsWindow);

fn spawn_command_window() -> impl Scene {
    bsn! [
        #CommandWindow
        CommandsWindow
        window()
        Node {
            top: percent(70)
        }
        DespawnOnExit<CommandsWindowOpen>(CommandsWindowOpen(true))
    ]
}

fn set_commands_window_open(mut state: ResMut<NextState<CommandsWindowOpen>>) {
    state.set(CommandsWindowOpen(true))
}

fn set_commands_window_closed(mut state: ResMut<NextState<CommandsWindowOpen>>) {
    state.set(CommandsWindowOpen(false))
}

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct CommandsWindowOpen(bool);