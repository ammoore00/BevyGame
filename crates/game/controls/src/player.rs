use bevy::prelude::*;
use common::{rotate_screen_space_to_facing, rotate_screen_space_to_movement, AppSystems, Facing, GameplaySystems, PausableSystems};
use input::InputReader;
use runtime::character::player::{AimFacing, AimFacingEvent, InputAttackEvent, InputJumpEvent, InputMoveEvent, Player};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (record_aim_input, record_movement_input, record_jump_input, record_attack_input)
                .in_set(GameplaySystems)
                .in_set(PausableSystems)
                .in_set(AppSystems::RecordInput),
        )
    );
}

fn record_aim_input(
    player_query: Query<(Entity, &AimFacing), With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    // TODO: Add keyboard and mouse support
    // TODO: Remappable controls

    // Add gamepad input if available
    if let Some(gamepad) = input_reader.gamepad() {
        let right_stick_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        let right_stick_y = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);

        // Apply deadzone
        let new_facing = if right_stick_x.abs() > 0.1 || right_stick_y.abs() > 0.1 {
            let aim_direction = Vec2::new(right_stick_x, right_stick_y);
            Some(Facing::from(rotate_screen_space_to_facing(aim_direction)))
        } else {
            None
        };

        if let Ok((aiming_entity, aim_facing)) = player_query.single()
            && new_facing != aim_facing.0
        {
            commands.trigger(AimFacingEvent::new(aiming_entity, new_facing))
        }
    }
}

fn record_movement_input(
    player_query: Query<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let player_entity = match player_query.single() {
        Ok(entity) => entity,
        Err(err) => {
            error!("Failed to find player entity: {}", err);
            return;
        },
    };

    // TODO: Better split of gamepad and keyboard controls
    // TODO: Remappable controls

    let mut intent = Vec3::ZERO;
    let mut toggle_sprint = false;

    if let Some(gamepad) = input_reader.gamepad()
    {
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

    commands.trigger(InputMoveEvent::new(player_entity, intent, toggle_sprint));
}

fn record_jump_input(
    player_query: Query<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let player_entity = match player_query.single() {
        Ok(entity) => entity,
        Err(err) => {
            error!("Failed to find player entity: {}", err);
            return;
        },
    };

    let mut jump = if let Some(gamepad) = input_reader.gamepad()
        && gamepad.just_pressed(GamepadButton::South)
    {
        true
    } else {
        input_reader.keyboard().just_pressed(KeyCode::Space)
    };

    if jump {
        commands.trigger(InputJumpEvent::new(player_entity));
    }
}

fn record_attack_input(
    player_query: Query<Entity, With<Player>>,
    input_reader: InputReader,
    mut commands: Commands,
) {
    let player_entity = match player_query.single() {
        Ok(entity) => entity,
        Err(err) => {
            error!("Failed to find player entity: {}", err);
            return;
        },
    };

    let attack = if let Some(gamepad) = input_reader.gamepad()
        && gamepad.just_pressed(GamepadButton::RightTrigger)
    {
        true
    } else {
        input_reader.mouse_buttons().just_pressed(MouseButton::Left)
    };

    if attack {
        commands.trigger(InputAttackEvent::new(player_entity, "player/basic_attack"));
    }
}