use crate::character::player::{AimFacing, AimFacingEvent, Player, PlayerAttackEvent};
use crate::character::stamina::Stamina;
use crate::character::state::tracking;
use crate::character::state::tracking::{is_in_movement_state, ActionStateEvent, ActionStateTracker};
use crate::character::Character;
use assets::action_states::{ActionState, ActionStateCapabilities, Attacking, Idle, Running, Sprinting, Walking};
use assets::resource::character::{AttackDefinition, AttackRegistry, AttackResource};
use bevy::prelude::*;
use common::{rotate_screen_space_to_facing, rotate_screen_space_to_movement, AppSystems, Facing, GameplaySystems, PausableSystems, WorldPosition};
use data::prelude::*;
use input::gamepad::GamepadRes;
use physics::{MovementController, PhysicsData};
use std::any::TypeId;

// TODO: Split this module into multiple files
//       One file should be in `input` to handle the actual player input
//       That file should emit events that this file consumes

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            // Normal Systems
            (record_aim_input,)
                .in_set(GameplaySystems)
                .in_set(PausableSystems)
                .in_set(AppSystems::RecordInput),
            camera_follow_player.in_set(AppSystems::Respond),
            // Exclusive Systems
            (record_action_input, record_player_movement_input)
                .chain()
                .in_set(GameplaySystems)
                .in_set(PausableSystems)
                .in_set(AppSystems::RecordInput),
        ),
    );
}

const COYOTE_TIME: f32 = 0.2;
const COYOTE_TIME_HEIGHT_THRESHOLD: f32 = 0.1;
const JUMP_VELOCITY: f32 = 2.75;

fn record_aim_input(
    gamepad_res: Option<Res<GamepadRes>>,
    gamepads: Query<&Gamepad>,
    aim_query: Query<(Entity, &AimFacing)>,
    mut commands: Commands,
) {
    // Add gamepad input if available
    if let Some(gamepad_res) = gamepad_res
        && let Ok(gamepad) = gamepads.get(gamepad_res.0)
    {
        let right_stick_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        let right_stick_y = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);

        // Apply deadzone
        let new_facing = if right_stick_x.abs() > 0.1 || right_stick_y.abs() > 0.1 {
            let aim_direction = Vec2::new(right_stick_x, right_stick_y);
            Some(Facing::from(rotate_screen_space_to_facing(aim_direction)))
        } else {
            None
        };

        if let Ok((aiming_entity, aim_facing)) = aim_query.single()
            && new_facing != aim_facing.0
        {
            commands.trigger(AimFacingEvent {
                entity: aiming_entity,
                facing: new_facing,
            })
        }
    }
}

fn record_player_movement_input(world: &mut World) {
    let mut intent = Vec3::ZERO;
    let mut is_jumping = false;
    let mut toggle_sprint = false;

    let input = world.resource::<ButtonInput<KeyCode>>();
    let gamepad_res = world.get_resource::<GamepadRes>();

    if let Some(gamepad_id) = gamepad_res.map(|r| r.0)
        && let Ok(Some(gamepad)) = world.get_entity(gamepad_id).map(|e| e.get::<Gamepad>())
    {
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);

        // Apply deadzone
        if left_stick_x.abs() > 0.1 || left_stick_y.abs() > 0.1 {
            intent.x += left_stick_x;
            intent.z -= left_stick_y;

            intent = rotate_screen_space_to_movement(intent);
        }

        if gamepad.just_pressed(GamepadButton::South) {
            is_jumping = true;
        }

        if gamepad.just_pressed(GamepadButton::LeftThumb) {
            toggle_sprint = true;
        }
    }

    if intent == Vec3::ZERO {
        // Collect directional input.
        if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            intent.z -= 1.0;
        }
        if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            intent.z += 1.0;
        }
        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            intent.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            intent.x += 1.0;
        }

        if input.just_pressed(KeyCode::Space) {
            is_jumping = true;
        }

        if input.just_pressed(KeyCode::ShiftLeft) {
            toggle_sprint = true;
        }

        // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
        intent = intent.normalize_or_zero();
        intent = rotate_screen_space_to_movement(intent);
    }

    let mut controller_query = world.query_filtered::<Entity, (
        With<Player>,
        With<Character>,
        With<MovementController>,
        With<PhysicsData>,
        With<WorldPosition>,
        With<ActionStateTracker>,
        With<ActionStateCapabilities>,
    )>();

    let entities: Vec<Entity> = controller_query.iter(world).collect();

    for entity in entities {
        // Get the current state
        let tracker = world.get::<ActionStateTracker>(entity).cloned().unwrap();
        let Some(prev_state) = tracking::get_state(entity, &tracker, world) else {
            warn!("Failed to get reflect component for entity {}", entity);
            continue;
        };

        // Check if the current state is movement
        let is_movement = is_in_movement_state(
            entity,
            &world.get::<ActionStateTracker>(entity).unwrap().clone(),
            world,
        );

        // Determine new state from movement intent
        let mut sprinting = {
            let controller = world.get::<MovementController>(entity).unwrap();
            controller.sprinting
        };

        let new_state: Box<dyn ActionState> = if intent.length() > 1e-6 {
            if intent.length() < 0.7 {
                sprinting = false;
                Box::new(Walking)
            } else {
                match (toggle_sprint, sprinting) {
                    // We aren't sprinting, and don't want to sprint
                    (false, false) => {
                        sprinting = false;
                        Box::new(Running)
                    }
                    // We aren't sprinting, and want to start sprinting
                    (false, true) => {
                        sprinting = true;
                        Box::new(Sprinting)
                    }
                    // We are sprinting, and want to keep sprinting
                    (true, false) => {
                        sprinting = true;
                        Box::new(Sprinting)
                    }
                    // We are sprinting, and want to stop sprinting
                    (true, true) => {
                        sprinting = false;
                        Box::new(Running)
                    }
                }
            }
        } else {
            sprinting = false;
            Box::new(Idle)
        };

        let state_capabilities = world.get::<ActionStateCapabilities>(entity).cloned().unwrap();

        // If the character state has changed
        if (*prev_state).type_id() != (*new_state).type_id() {
            // Attempt to create a state transition event
            let should_sprint = (*new_state).type_id() == TypeId::of::<Sprinting>();
            if let Ok(event) =
                ActionStateEvent::try_new(entity, &state_capabilities, new_state, prev_state)
            {
                world.trigger(event);
                sprinting = should_sprint;
            }
        }

        // Update the controller's intent
        if let Some(mut controller) = world.get_mut::<MovementController>(entity) {
            controller.sprinting = sprinting;
            if is_movement {
                controller.intent = intent;
            } else {
                controller.intent = Vec3::ZERO;
            }
        }

        // Handle jumping
        let physics = world.get::<PhysicsData>(entity).unwrap();
        let position = world.get::<WorldPosition>(entity).unwrap();

        if let PhysicsData::Kinematic {
            time_since_grounded,
            last_grounded_height,
            ..
        } = *physics
            && time_since_grounded < COYOTE_TIME
            && position.as_vec3().y < last_grounded_height + COYOTE_TIME_HEIGHT_THRESHOLD
            && is_jumping
            && let Some(mut controller) = world.get_mut::<MovementController>(entity)
        {
            controller.intent.y = JUMP_VELOCITY;
        }
    }
}

fn record_action_input(world: &mut World) {
    let gamepad = world.get_resource::<GamepadRes>().map(|r| r.0);

    let player = {
        let mut query = world.query_filtered::<Entity, With<Player>>();
        query.single(world).ok()
    };

    let (_player, gamepad_id) = match (player, gamepad) {
        (Some(p), Some(g)) => (p, g),
        _ => return,
    };

    let gamepad = world
        .get_entity(gamepad_id)
        .unwrap()
        .get::<Gamepad>()
        .unwrap();
    let attack = gamepad.just_pressed(GamepadButton::RightTrigger);

    let mut player_query = world.query_filtered::<Entity, (
        With<Player>,
        With<Character>,
        With<Facing>,
        With<Stamina>,
        With<ActionStateTracker>,
        With<ActionStateCapabilities>,
    )>();
    let player = player_query.single(world).unwrap();

    let state_capabilities = world.get::<ActionStateCapabilities>(player).cloned().unwrap();

    // 2. Check if it's a movement state (this takes &mut World)
    let is_movement = is_in_movement_state(
        player,
        &world.get::<ActionStateTracker>(player).unwrap().clone(),
        world,
    );

    let is_idle = world
        .query_filtered::<Entity, With<Idle>>()
        .get(world, player)
        .is_ok();

    let prev_state = {
        let state_tracker = world.get::<ActionStateTracker>(player).cloned().unwrap();

        let Some(prev_state) = tracking::get_state(player, &state_tracker, world) else {
            error!("Failed to get reflect component for entity {}", player);
            return;
        };
        prev_state
    };

    let stamina = world.get::<Stamina>(player).cloned().unwrap();

    let mut aim_facing_query = world.query_filtered::<Entity, With<AimFacing>>();
    let aim_facing = aim_facing_query.single(world).unwrap();
    let aim_facing = world.get::<AimFacing>(aim_facing).cloned().unwrap();

    // TODO: Move this logic into attack module
    if attack && (is_movement || is_idle) && stamina.current > 0 {
        let facing = {
            let mut facing = world.get_mut::<Facing>(player).unwrap();
            if let Some(aim_facing) = aim_facing.0 {
                *facing = aim_facing;
            }
            *facing
        };

        let attack_loc: ResourceLocation<AttackResource> = "player/basic_attack".parse().unwrap();

        world.trigger(PlayerAttackEvent {
            entity: player,
            facing,
            attack: attack_loc.clone(),
        });

        let attack_registry = world.resource::<AttackRegistry>();
        let attack_assets = world.resource::<Assets<AttackDefinition>>();

        let Some(attack_handle) = attack_registry.get(&attack_loc) else {
            error!("Failed to find attack resource for location: {}", attack_loc);
            return;
        };
        let Some(attack) = attack_assets.get(attack_handle) else {
            error!("Failed to find attack asset for handle: {:?}", attack_handle);
            return;
        };

        match ActionStateEvent::try_new(
            player,
            &state_capabilities,
            Box::new(Attacking::new(&attack_loc, *attack.duration())),
            prev_state,
        ) {
            Ok(event) => world.trigger(event),
            Err(_) => {
                error!("Failed to create CharacterStateEvent for Attacking state");
            }
        }
    }
}

fn camera_follow_player(
    player_query: Query<&mut Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    // Update camera position to match player position
    camera_transform.translation = player_transform.translation * Vec3::new(1.0, 1.0, 0.0);
}