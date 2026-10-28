use crate::{PlayerControlSystems, last_input_gamepad, last_input_keyboard};
use bevy::prelude::*;
use common::rotate_screen_space_to_movement;
use input::gamepad::{GamepadStick, get_stick_with_deadzone};
use input::{InputReader, LastInputMode};
use runtime::characters::player::MoveInputEvent;
use runtime::prelude::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            movement_input_gamepad.run_if(last_input_gamepad),
            movement_input_keyboard.run_if(last_input_keyboard),
        )
            .in_set(PlayerControlSystems),
    );
}

fn movement_input_gamepad(
    player: Single<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let mut intent = Vec3::ZERO;
    let mut toggle_sprint = false;

    if let Some(gamepad) = input_reader.gamepad()
        && **input_reader.last_input_mode() == LastInputMode::Gamepad
    {
        let left_stick = get_stick_with_deadzone(gamepad, GamepadStick::Left);

        intent.x += left_stick.x;
        intent.z -= left_stick.y;
        intent = rotate_screen_space_to_movement(intent);

        if gamepad.just_pressed(GamepadButton::LeftThumb) {
            toggle_sprint = true;
        }
    }

    commands.trigger(MoveInputEvent::new(player.entity(), intent, toggle_sprint));
}

fn movement_input_keyboard(
    player: Single<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let mut intent = Vec3::ZERO;
    let mut toggle_sprint = false;

    let keyboard = input_reader.keyboard();
    if **input_reader.last_input_mode() == LastInputMode::MouseAndKeyboard {
        // Collect directional input.
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            intent.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            intent.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            intent.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            intent.x += 1.0;
        }

        if keyboard.just_pressed(KeyCode::ShiftLeft) {
            toggle_sprint = true;
        }

        // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
        intent = intent.normalize_or_zero();
        intent = rotate_screen_space_to_movement(intent);
    }

    commands.trigger(MoveInputEvent::new(player.entity(), intent, toggle_sprint));
}
