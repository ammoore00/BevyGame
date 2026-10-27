use bevy::prelude::*;
use common::{rotate_screen_space_to_facing, rotate_screen_space_to_movement, AppSystems, Facing, GameInputSystems, GameplaySystems, PausableSystems};
use input::gamepad::{GamepadStick, get_stick_with_deadzone};
use input::{InputReader, LastInputMode};
use runtime::LevelLoadedSystems;
use runtime::characters::player::{
    AimInputEvent, AttackInputEvent, DodgeInputEvent, JumpInputEvent, MoveInputEvent, Player,
};

// TODO: Remappable controls
// TODO: Split keyboard and gamepad input checks into separate systems

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            record_aim_input,
            record_attack_input,
            record_dodge_input,
            record_jump_input,
            record_movement_input,
        )
            .in_set(GameplaySystems)
            .in_set(PausableSystems)
            .in_set(LevelLoadedSystems)
            .in_set(GameInputSystems)
            .in_set(AppSystems::RecordInput),
    );
}

fn record_aim_input(
    player: Single<Entity, With<Player>>,
    window: Query<&Window>,
    mut input_reader: InputReader,
    mut commands: Commands,
) {
    let player_entity = player.into_inner();
    let window = window.single().expect("Failed to find window");

    if let Some(gamepad) = input_reader.gamepad()
        && **input_reader.last_input_mode() == LastInputMode::Gamepad
    {
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
        commands.trigger(AimInputEvent::new(player_entity, new_facing));
    }

    if **input_reader.last_input_mode() == LastInputMode::MouseAndKeyboard {
        let Some(last_cursor) = input_reader.mouse_cursor_mut().read().last() else {
            return;
        };
        
        let window_center =
            Vec2::new(window.resolution.width(), window.resolution.height()) / 2.0;
        
        let mut delta = last_cursor.position - window_center;
        delta.y /= 0.707; // Scale to isometric
        
        let new_facing = Some(Facing::from(rotate_screen_space_to_facing(delta, false)));
        
        commands.trigger(AimInputEvent::new(player_entity, new_facing));
    }
}

fn record_movement_input(
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

// TODO: Consolidate these event trigger functions to reduce code duplication

fn record_jump_input(
    player: Single<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let jump = if let Some(gamepad) = input_reader.gamepad()
        && **input_reader.last_input_mode() == LastInputMode::Gamepad
        && gamepad.just_pressed(GamepadButton::South)
    {
        true
    } else {
        input_reader.keyboard().just_pressed(KeyCode::Space)
            && **input_reader.last_input_mode() == LastInputMode::MouseAndKeyboard
    };

    if jump {
        commands.trigger(JumpInputEvent::new(player.entity()));
    }
}

fn record_attack_input(
    player: Single<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let attack = if let Some(gamepad) = input_reader.gamepad()
        && **input_reader.last_input_mode() == LastInputMode::Gamepad
        && gamepad.just_pressed(GamepadButton::RightTrigger)
    {
        true
    } else {
        input_reader.mouse_buttons().just_pressed(MouseButton::Left)
            && **input_reader.last_input_mode() == LastInputMode::MouseAndKeyboard
    };

    if attack {
        commands.trigger(AttackInputEvent::new(
            player.entity(),
            "player/basic_attack",
        ));
    }
}

fn record_dodge_input(
    player: Single<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let dodge = if let Some(gamepad) = input_reader.gamepad()
        && **input_reader.last_input_mode() == LastInputMode::Gamepad
        && gamepad.just_pressed(GamepadButton::East)
    {
        true
    } else {
        input_reader.mouse_buttons().just_pressed(MouseButton::Middle)
            && **input_reader.last_input_mode() == LastInputMode::MouseAndKeyboard
    };
    
    if dodge {
        commands.trigger(DodgeInputEvent::new(player.entity()));
    }
}
