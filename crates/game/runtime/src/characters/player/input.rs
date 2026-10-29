use crate::characters::attack::{AttackValidated, TryAttackEvent};
use crate::characters::dodge::{DodgeValidated, TryDodgeEvent};
use crate::characters::player::{AimFacing, Player};
use crate::characters::state::{ActionStateTracker, TrySetStateEvent};
use assets::action_states::{
    ActionState, ActionStateCapabilities, Idle, Running, Sprinting, Walking,
};
use assets::resource::characters::AttackResource;
use bevy::prelude::*;
use common::{AppSystems, Facing, WorldPosition};
use data::prelude::*;
use physics::{ApplyImpulse, Impulse, KinematicData, MovementController, PhysicsData};
use std::any::TypeId;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        camera_follow_player.in_set(AppSystems::Respond),
    );

    app.add_observer(on_aim_input);
    app.add_observer(on_movement_input);

    app.add_observer(on_attack_input);
    app.add_observer(on_attack_validated);

    app.add_observer(on_dodge_input);
    app.add_observer(on_dodge_validated);

    app.add_observer(on_jump_input);
}

const COYOTE_TIME: f32 = 0.2;
const COYOTE_TIME_HEIGHT_THRESHOLD: f32 = 0.1;
const JUMP_VELOCITY: f32 = 12.0;

#[derive(EntityEvent, derive_new::new)]
pub struct MoveInputEvent {
    entity: Entity,
    intent: Vec3,
    toggle_sprint: bool,
}

fn on_movement_input(
    event: On<MoveInputEvent>,
    mut player_query: Query<
        (&mut MovementController, &ActionStateTracker),
        (With<Player>, With<ActionStateCapabilities>),
    >,
    mut commands: Commands,
) {
    let Ok((mut controller, tracker)) = player_query.get_mut(event.entity) else {
        error!("Failed to get player movement info");
        return;
    };

    let new_state: Box<dyn ActionState> = if event.intent.length() > 1e-6 {
        if event.intent.length() < 0.7 {
            Box::new(Walking)
        } else {
            if event.toggle_sprint ^ controller.sprinting {
                Box::new(Sprinting)
            } else {
                Box::new(Running)
            }
        }
    } else {
        Box::new(Idle)
    };

    let prev_state_id = tracker.state_type_id();
    let new_state_id = (*new_state).type_id();

    // If the characters state has changed
    if prev_state_id != new_state_id {
        let should_sprint = (*new_state).type_id() == TypeId::of::<Sprinting>();

        // Set the state to the new state, with handling depending on the result
        let event = TrySetStateEvent::new(event.entity, new_state).with_callback(
            move |entity, mut commands, result| {
                // If the state was not set, do nothing
                if result.is_err() {
                    return;
                }

                // If the state was set, update the controller's sprinting state as appropriate
                commands
                    .entity(entity)
                    .queue(move |mut entity_world: EntityWorldMut| {
                        if let Some(mut controller) = entity_world.get_mut::<MovementController>() {
                            controller.sprinting = should_sprint;
                        } else {
                            error!("Failed to get MovementController component for player");
                        }
                    });
            },
        );

        commands.trigger(event);
    }

    let prev_y = controller.intent.y;

    // Update the controller's intent
    if tracker.is_movement() {
        controller.intent = event.intent;
    } else {
        controller.intent = Vec3::ZERO;
    }

    // Vertical intent (aka jumping) is handled separately, and we don't want to reset it
    controller.intent.y = prev_y;
}

#[derive(EntityEvent)]
pub struct JumpInput(pub Entity);

#[derive(EntityEvent)]
pub struct JumpInputConfirmed(pub Entity);

// TODO: Convert jumping to use its own state
fn on_jump_input(
    event: On<JumpInput>,
    mut player_query: Query<
        (
            Entity,
            &ActionStateTracker,
            &PhysicsData,
            &WorldPosition,
            Option<&Idle>,
        ),
        (With<Player>, With<ActionStateCapabilities>),
    >,
    mut commands: Commands,
) {
    let Ok((entity, tracker, physics, position, idle)) = player_query.get_mut(event.0) else {
        error!("Failed to get player movement info");
        return;
    };

    if let None = idle
        && !tracker.is_movement()
    {
        info!("Cannot jump, player is not in valid state!");
        return;
    }

    let PhysicsData::Kinematic(KinematicData {
        time_since_grounded,
        last_grounded_height,
        ..
    }) = *physics
    else {
        panic!("Player assigned non-kinematic physics data! This is a bug!")
    };

    if time_since_grounded < COYOTE_TIME
        && position.as_vec3().y <= last_grounded_height
        && position.as_vec3().y >= last_grounded_height - COYOTE_TIME_HEIGHT_THRESHOLD
    {
        commands.trigger(ApplyImpulse {
            entity,
            impulse: Impulse::from(Vec3::Y * JUMP_VELOCITY),
        });
        
        commands.entity(entity).trigger(JumpInputConfirmed);
    } else {
        info!("Cannot jump, player is not grounded!");
    }
}

#[derive(EntityEvent)]
pub struct AttackInput(pub Entity);

fn on_attack_input(
    _: On<AttackInput>,
    player: Single<(Entity, &mut Facing), (With<Player>, With<Children>)>,
    aim_facing: Single<(&AimFacing, &ChildOf)>,
    mut commands: Commands,
) {
    let (player_entity, mut facing) = player.into_inner();
    let (aim_facing, child_of) = aim_facing.into_inner();

    if child_of.0 != player_entity {
        error!("AimFacing is not a child of the player entity!");
        return;
    }

    let facing = {
        if let Some(aim_facing) = aim_facing.0 {
            *facing = aim_facing;
        }
        *facing
    };

    let attack_loc: ResourceLocation<AttackResource> = "player/basic_attack".parse().unwrap();
    let event = TryAttackEvent::new(player_entity, facing, attack_loc);
    commands.trigger(event);
}

#[derive(EntityEvent)]
pub struct AttackInputConfirmed(pub Entity);

fn on_attack_validated(
    event: On<AttackValidated>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    if event.entity() == player.entity() {
        commands
            .entity(event.entity())
            .trigger(AttackInputConfirmed);
    }
}

#[derive(EntityEvent, Debug, Clone)]
pub struct DodgeInput(pub Entity);

fn on_dodge_input(
    _: On<DodgeInput>,
    player: Single<(Entity, &MovementController, &mut Facing), (With<Player>, With<Children>)>,
    aim_facing: Single<(&AimFacing, &ChildOf)>,
    mut commands: Commands,
) {
    let (player_entity, movement_controller, mut facing) = player.into_inner();

    // Base dodge direction on current movement
    let mut direction = movement_controller.intent;
    direction.y = 0.0;

    // If the player is not moving, dodge in the direction of their aiming
    if direction == Vec3::ZERO {
        let (aim_facing, child_of) = aim_facing.into_inner();

        if child_of.0 != player_entity {
            error!("AimFacing is not a child of the player entity!");
            return;
        }

        // If the player is not aiming, dodge in the direction of their facing
        let facing = {
            if let Some(aim_facing) = aim_facing.0 {
                *facing = aim_facing;
            }
            *facing
        };

        direction = Vec3::from(facing);
    }

    commands.trigger(TryDodgeEvent::new(player_entity, direction));
}

#[derive(EntityEvent)]
pub struct DodgeInputConfirmed(pub Entity);

fn on_dodge_validated(
    event: On<DodgeValidated>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    if event.entity() == player.entity() {
        commands.entity(event.entity()).trigger(DodgeInputConfirmed);
    }
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
pub struct AimInputEvent {
    entity: Entity,
    facing: Option<Facing>,
}

fn on_aim_input(
    event: On<AimInputEvent>,
    aim_facing: Single<(&mut AimFacing, &mut Sprite, &mut Visibility, &ChildOf)>,
) {
    let (mut aim_facing, mut sprite, mut visibility, child_of) = aim_facing.into_inner();

    if child_of.0 != event.entity {
        error!("Aim facing event received for wrong entity!");
        return;
    }

    if event.facing == aim_facing.0 {
        return;
    }

    if let Some(new_facing) = event.facing {
        aim_facing.0 = Some(new_facing);
        visibility
            .set(Box::new(Visibility::Inherited))
            .expect("Failed to set visibility");
        sprite.texture_atlas.as_mut().unwrap().index = new_facing as usize;
    } else {
        aim_facing.0 = None;
        visibility
            .set(Box::new(Visibility::Hidden))
            .expect("Failed to set visibility");
    }
}

/// Update the camera position to match the player position
// TODO: Move this somewhere more appropriate
fn camera_follow_player(
    player_transform: Single<&mut Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    camera.translation = player_transform.translation * Vec3::new(1.0, 1.0, 0.0);
}
