use crate::window;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use common::{marker, GameState, Pause};

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