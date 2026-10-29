use crate::buffer::{InputKind, InsertBuffer, on_input_success};
use crate::{PlayerControlSystems, last_input_gamepad, last_input_keyboard};
use bevy::prelude::*;
use input::InputReader;
use runtime::characters::player::{AttackInputConfirmed, DodgeInputConfirmed, JumpInputConfirmed};

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

    app.add_observer(on_input_success::<AttackInputConfirmed, { InputKind::Attack }>);
    app.add_observer(on_input_success::<DodgeInputConfirmed, { InputKind::Dodge }>);
    app.add_observer(on_input_success::<JumpInputConfirmed, { InputKind::Jump }>);
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
    input_reader: InputReader,
    mut commands: Commands,
    input: impl Into<InputButton>,
    input_kind: InputKind,
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
        commands.trigger(InsertBuffer::new(input_kind));
    }
}

macro_rules! input {
    ($fn_name:ident, $input:expr, $input_kind:expr) => {
        fn $fn_name(input_reader: InputReader, commands: Commands) {
            action_input(input_reader, commands, $input, $input_kind);
        }
    };
}

// TODO: Remappable controls

input!(jump_gamepad, GamepadButton::South, InputKind::Jump);
input!(jump_keyboard, KeyCode::Space, InputKind::Jump);

input!(
    attack_gamepad,
    GamepadButton::RightTrigger,
    InputKind::Attack
);
input!(attack_keyboard, MouseButton::Left, InputKind::Attack);

input!(dodge_gamepad, GamepadButton::LeftTrigger, InputKind::Dodge);
input!(dodge_keyboard, MouseButton::Middle, InputKind::Dodge);
