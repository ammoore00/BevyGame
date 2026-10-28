use crate::{PlayerControlSystems, last_input_gamepad, last_input_keyboard};
use bevy::prelude::*;
use input::InputReader;
use runtime::characters::player::{AttackInputEvent, DodgeInputEvent, JumpInputEvent, Player};

pub(super) fn plugin(app: &mut App) {
    let gamepad_inputs = (attack_gamepad, dodge_gamepad, jump_gamepad);
    let keyboard_inputs = (attack_keyboard, dodge_keyboard, jump_keyboard);

    app.add_systems(
        Update,
        (
            gamepad_inputs.run_if(last_input_gamepad),
            keyboard_inputs.run_if(last_input_keyboard),
        )
            .in_set(PlayerControlSystems),
    );
}

enum InputButton {
    Gamepad(GamepadButton),
    Keyboard(KeyCode),
    Mouse(MouseButton),
}
impl From<GamepadButton> for InputButton {
    fn from(value: GamepadButton) -> Self {
        Self::Gamepad(value)
    }
}
impl From<KeyCode> for InputButton {
    fn from(value: KeyCode) -> Self {
        Self::Keyboard(value)
    }
}
impl From<MouseButton> for InputButton {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}

fn action_input(
    player: Single<Entity, With<Player>>,
    input_reader: InputReader,
    commands: Commands,
    input: impl Into<InputButton>,
    event_callback: Box<dyn FnOnce(Commands, Entity)>,
) {
    let input = input.into();

    let trigger = match input {
        InputButton::Gamepad(gamepad_button) => input_reader
            .gamepad()
            .map(|gamepad| gamepad.just_pressed(gamepad_button))
            .unwrap_or_default(),
        InputButton::Keyboard(keycode) => input_reader.keyboard().just_pressed(keycode),
        InputButton::Mouse(mouse_button) => input_reader.mouse_buttons().just_pressed(mouse_button),
    };

    if trigger {
        event_callback(commands, player.entity());
    }
}

macro_rules! input {
    ($fn_name:ident, $input:expr, $event_callback:expr) => {
        fn $fn_name(
            player: Single<Entity, With<Player>>,
            input_reader: InputReader,
            commands: Commands,
        ) {
            action_input(
                player,
                input_reader,
                commands,
                $input,
                Box::new(|mut commands, entity| {
                    commands.entity(entity).trigger($event_callback);
                }),
            );
        }
    };
}

// TODO: Remappable controls

input!(jump_gamepad, GamepadButton::South, JumpInputEvent);
input!(jump_keyboard, KeyCode::Space, JumpInputEvent);

input!(
    attack_gamepad,
    GamepadButton::RightTrigger,
    AttackInputEvent
);
input!(attack_keyboard, MouseButton::Left, AttackInputEvent);

input!(dodge_gamepad, GamepadButton::LeftTrigger, DodgeInputEvent);
input!(dodge_keyboard, MouseButton::Middle, DodgeInputEvent);
