use bevy::prelude::*;
use common::{AppSystems, GameInputSystems, GameplaySystems, PausableSystems};
use input::{InputReader, LastInputMode};
use runtime::LevelLoadedSystems;

mod aim;
mod buffer;
mod movement;
mod action;

pub struct ControlsPlugin;
impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            aim::plugin,
            buffer::plugin,
            movement::plugin,
            action::plugin,
        ));

        app.configure_sets(
            Update,
            PlayerControlSystems
                .in_set(GameplaySystems)
                .in_set(PausableSystems)
                .in_set(LevelLoadedSystems)
                .in_set(GameInputSystems)
                .in_set(AppSystems::RecordInput),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct PlayerControlSystems;

fn last_input_gamepad(input_reader: InputReader) -> bool {
    **input_reader.last_input_mode() != LastInputMode::Gamepad
}

fn last_input_keyboard(input_reader: InputReader) -> bool {
    **input_reader.last_input_mode() != LastInputMode::MouseAndKeyboard
}
