use crate::{PlayerControlSystems, last_input_gamepad, last_input_keyboard};
use bevy::prelude::*;
use common::{rotate_screen_space_to_facing, Facing};
use input::InputReader;
use runtime::characters::player::AimInputEvent;
use runtime::prelude::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            aim_input_gamepad.run_if(last_input_gamepad),
            aim_input_keyboard.run_if(last_input_keyboard),
        )
            .in_set(PlayerControlSystems),
    );
}

fn aim_input_gamepad(
    player: Single<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let Some(gamepad) = input_reader.gamepad() else {
        error!("Last input was gamepad, but no gamepad resource was found!");
        return;
    };

    let right_stick_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
    let right_stick_y = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);

    // Apply deadzone
    let new_facing = if right_stick_x.abs() > 0.1 || right_stick_y.abs() > 0.1 {
        let aim_direction = Vec2::new(right_stick_x, right_stick_y);
        Some(Facing::from(rotate_screen_space_to_facing(
            aim_direction,
            true,
        )))
    } else {
        None
    };
    commands.trigger(AimInputEvent::new(player.into_inner(), new_facing));
}

fn aim_input_keyboard(
    player: Single<Entity, With<Player>>,
    window: Single<&Window>,
    mut input_reader: InputReader,
    mut commands: Commands,
) {
    let Some(last_cursor) = input_reader.mouse_cursor_mut().read().last() else {
        return;
    };

    let window_center = Vec2::new(window.resolution.width(), window.resolution.height()) / 2.0;

    let mut delta = last_cursor.position - window_center;
    delta.y /= 0.707; // Scale to isometric

    let new_facing = Some(Facing::from(rotate_screen_space_to_facing(delta, false)));
    commands.trigger(AimInputEvent::new(player.into_inner(), new_facing));
}
