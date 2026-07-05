use bevy::prelude::*;
use common::{rotate_screen_space_to_facing, rotate_screen_space_to_movement, AppSystems, Facing, GameplaySystems, PausableSystems, ScreenCoords, WorldPosition};
use input::{InputReader, LastInputMode};
use runtime::character::player::{AimInputEvent, AttackInputEvent, JumpInputEvent, MoveInputEvent, Player};
use runtime::LevelLoadedSystems;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (record_aim_input, record_movement_input, record_jump_input, record_attack_input)
                .in_set(GameplaySystems)
                .in_set(PausableSystems)
                .in_set(LevelLoadedSystems)
                .in_set(AppSystems::RecordInput),
        )
    );
}

fn record_aim_input(
    player_query: Query<
        (
            Entity,
            &WorldPosition,
        ),
        With<Player>,
    >,
    window:Query<&Window>,
    mut input_reader: InputReader,
    last_input_mode: Res<LastInputMode>,
    mut commands: Commands,
) {
    // TODO: Add keyboard and mouse support
    // TODO: Remappable controls

    let (player_entity, position) = player_query.single().expect("Failed to find player entity");
    let window = window.single().expect("Failed to find window");

    if let Some(gamepad) = input_reader.gamepad() {
        let right_stick_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        let right_stick_y = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);

        // Apply deadzone
        let new_facing = if right_stick_x.abs() > 0.1 || right_stick_y.abs() > 0.1 {
            let aim_direction = Vec2::new(right_stick_x, right_stick_y);
            Some(Facing::from(rotate_screen_space_to_facing(aim_direction, true)))
        } else {
            None
        };
        commands.trigger(AimInputEvent::new(player_entity, new_facing));
    }

    if *last_input_mode == LastInputMode::MouseAndKeyboard {
        let Some(last_cursor) = input_reader.cursor_mut().read().last() else {
            return;
        };

        let cursor_position = last_cursor.position - Vec2::from((window.resolution.width(), window.resolution.height())) / 2.0;

        let mut delta = cursor_position - ScreenCoords::from(position.0.clone()).xz();
        delta.y *= 0.707; // Scale to isometric
        let new_facing = Some(Facing::from(rotate_screen_space_to_facing(delta, false)));

        commands.trigger(AimInputEvent::new(player_entity, new_facing));
    }
}

fn record_movement_input(
    player_query: Query<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let player_entity = player_query.single()
        .expect("Failed to find player entity");

    // TODO: Better split of gamepad and keyboard controls
    // TODO: Remappable controls

    let mut intent = Vec3::ZERO;
    let mut toggle_sprint = false;

    if let Some(gamepad) = input_reader.gamepad() {
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);

        // Apply deadzone
        if left_stick_x.abs() > 0.1 || left_stick_y.abs() > 0.1 {
            intent.x += left_stick_x;
            intent.z -= left_stick_y;

            intent = rotate_screen_space_to_movement(intent);
        }

        if gamepad.just_pressed(GamepadButton::LeftThumb) {
            toggle_sprint = true;
        }
    }

    let keyboard = input_reader.keyboard();
    if intent == Vec3::ZERO {
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

    commands.trigger(MoveInputEvent::new(player_entity, intent, toggle_sprint));
}

fn record_jump_input(
    player_query: Query<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let player_entity = player_query.single()
        .expect("Failed to find player entity");

    let jump = if let Some(gamepad) = input_reader.gamepad()
        && gamepad.just_pressed(GamepadButton::South)
    {
        true
    } else {
        input_reader.keyboard().just_pressed(KeyCode::Space)
    };

    if jump {
        commands.trigger(JumpInputEvent::new(player_entity));
    }
}

fn record_attack_input(
    player_query: Query<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let player_entity = player_query.single()
        .expect("Failed to find player entity");

    let attack = if let Some(gamepad) = input_reader.gamepad()
        && gamepad.just_pressed(GamepadButton::RightTrigger)
    {
        true
    } else {
        input_reader.mouse_buttons().just_pressed(MouseButton::Left)
    };

    if attack {
        commands.trigger(AttackInputEvent::new(player_entity, "player/basic_attack"));
    }
}