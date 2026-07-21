use crate::character::attack::AttackEvent;
use crate::character::player::{AimFacing, Player};
use crate::character::stamina::Stamina;
use crate::character::state::{ActionStateTracker, TrySetStateEvent};
use assets::action_states::{ActionState, ActionStateCapabilities, Attacking, Idle, Running, Sprinting, Walking};
use assets::resource::characters::AttackResource;
use bevy::prelude::*;
use common::{AppSystems, Facing, WorldPosition};
use data::prelude::*;
use physics::{KinematicData, MovementController, PhysicsData};
use std::any::TypeId;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        camera_follow_player.in_set(AppSystems::Respond)
    );

    app.add_observer(on_movement_input);
    app.add_observer(on_jump_input);
    app.add_observer(on_attack_input);
    app.add_observer(on_aim_input);
}

const COYOTE_TIME: f32 = 0.2;
const COYOTE_TIME_HEIGHT_THRESHOLD: f32 = 0.1;
const JUMP_VELOCITY: f32 = 2.75;

#[derive(EntityEvent, derive_new::new)]
pub struct MoveInputEvent {
    entity: Entity,
    intent: Vec3,
    toggle_sprint: bool,
}

fn on_movement_input(
    event: On<MoveInputEvent>,
    mut player_query: Query<
        (
            &mut MovementController,
            &ActionStateTracker,
        ),
        (
            With<Player>,
            With<ActionStateCapabilities>,
        )
    >,
    mut commands: Commands,
) {
    let Ok((
        mut controller,
        tracker,
    )) = player_query.get_mut(event.entity) else {
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
        let event = TrySetStateEvent::new(event.entity, new_state)
            .with_callback(move |entity, mut commands, result| {
                // If the state was not set, do nothing
                if result.is_err() {
                    return;
                }

                // If the state was set, update the controller's sprinting state as appropriate
                commands.entity(entity).queue(
                    move |mut entity_world: EntityWorldMut| {
                        if let Some(mut controller) = entity_world.get_mut::<MovementController>() {
                            controller.sprinting = should_sprint;
                        } else {
                            error!("Failed to get MovementController component for player");
                        }
                    });
            });

        commands.trigger(event);
    }

    // Update the controller's intent
    if tracker.is_movement() {
        controller.intent = event.intent;
    } else {
        controller.intent = Vec3::ZERO;
    }
}

#[derive(EntityEvent, derive_new::new)]
pub struct JumpInputEvent {
    entity: Entity,
}

// TODO: Convert jumping to use its own state
fn on_jump_input(
    event: On<JumpInputEvent>,
    mut player_query: Query<
        (
            &mut MovementController,
            &ActionStateTracker,
            &PhysicsData,
            &WorldPosition,
            Option<&Idle>,
        ),
        (
            With<Player>,
            With<ActionStateCapabilities>,
        )
    >,
) {
    let Ok((
        mut controller,
        tracker,
        physics,
        position,
        idle,
    )) = player_query.get_mut(event.entity) else {
        error!("Failed to get player movement info");
        return;
    };

    if let None = idle && !tracker.is_movement() {
        info!("Cannot jump, player is not in valid state!");
        return;
    }

    if let PhysicsData::Kinematic(KinematicData {
        time_since_grounded,
        last_grounded_height,
        ..
    }) = *physics {
        if time_since_grounded < COYOTE_TIME
            && position.as_vec3().y <= last_grounded_height
            && position.as_vec3().y >= last_grounded_height - COYOTE_TIME_HEIGHT_THRESHOLD
        {
            info!("Jumping!");
            controller.intent.y = JUMP_VELOCITY;
        } else {
            info!("Cannot jump, player is not grounded!");
        }
    } else { panic!("Player assigned non-kinematic physics data! This is a bug!") }
}

#[derive(EntityEvent)]
pub struct AttackInputEvent {
    entity: Entity,
    _attack: ResourceLocation<AttackResource>,
}
impl AttackInputEvent {
    pub fn new(entity: Entity, attack_loc: impl AsRef<str>) -> Self {
        Self {
            entity,
            _attack: loc::<AttackResource>(attack_loc.as_ref())
                .unwrap_or_else(|_| panic!("Failed to parse attack resource location: {}", attack_loc.as_ref())),
        }
    }
}

fn on_attack_input(
    event: On<AttackInputEvent>,
    mut player_query: Query<
        (
            Entity,
            &mut Facing,
            &Stamina,
        ),
        (
            With<Player>,
            With<Children>,
        )
    >,
    aim_facing_query: Query<(&AimFacing, &ChildOf)>,
    attack_registry: SystemRegistry<AttackResource>,
    mut commands: Commands,
) {
    let (
        player_entity,
        mut facing,
        stamina,
    ) = player_query.get_mut(event.entity)
        .expect("Failed to get player entity");
    let (aim_facing, child_of) = aim_facing_query.single().expect("Failed to get aim facing");

    if child_of.0 != player_entity {
        error!("AimFacing is not a child of the player entity!");
        return;
    }

    // TODO: Make this check better
    if stamina.current > 0 {
        // TODO: Move this into the characters attack event
        let facing = {
            if let Some(aim_facing) = aim_facing.0 {
                *facing = aim_facing;
            }
            *facing
        };

        let attack_loc: ResourceLocation<AttackResource> = "player/basic_attack".parse().unwrap();

        commands.trigger(AttackEvent::new(player_entity, facing, attack_loc.clone()));

        let Some(attack) = attack_registry.get_asset(&attack_loc) else {
            error!("Attack resource {} does not exist!", attack_loc);
            return;
        };

        let attack_state = Box::new(Attacking::new(&attack_loc, *attack.duration()));
        commands.trigger(TrySetStateEvent::new(player_entity, attack_state));
    }
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
pub struct AimInputEvent {
    entity: Entity,
    facing: Option<Facing>,
}

fn on_aim_input(
    event: On<AimInputEvent>,
    mut query: Query<(&mut AimFacing, &mut Sprite, &mut Visibility, &ChildOf)>,
) {
    let Ok((mut aim_facing, mut sprite, mut visibility, child_of)) = query.single_mut() else {
        error!("Failed to get aim facing query!");
        return;
    };

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