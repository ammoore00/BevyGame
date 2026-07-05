use crate::character::player::{AimFacing, Player, PlayerAttackEvent};
use crate::character::stamina::Stamina;
use crate::character::state::{ActionStateTracker, TrySetStateEvent};
use assets::action_states::{ActionState, ActionStateCapabilities, Attacking, Idle, Running, Sprinting, Walking};
use assets::resource::character::{AttackResource};
use bevy::prelude::*;
use common::{AppSystems, Facing, WorldPosition};
use data::prelude::*;
use physics::{MovementController, PhysicsData};
use std::any::TypeId;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            // Normal Systems
            camera_follow_player.in_set(AppSystems::Respond),
        ),
    );

    app.add_observer(on_movement);
    app.add_observer(on_jump);
    app.add_observer(on_attack);
}

const COYOTE_TIME: f32 = 0.2;
const COYOTE_TIME_HEIGHT_THRESHOLD: f32 = 0.1;
const JUMP_VELOCITY: f32 = 2.75;

#[derive(EntityEvent, derive_new::new)]
pub struct InputMoveEvent {
    entity: Entity,
    intent: Vec3,
    toggle_sprint: bool,
}

#[derive(EntityEvent, derive_new::new)]
pub struct InputJumpEvent {
    entity: Entity,
}

#[derive(EntityEvent)]
pub struct InputAttackEvent {
    entity: Entity,
    _attack: ResourceLocation<AttackResource>,
}
impl InputAttackEvent {
    pub fn new(entity: Entity, attack_loc: impl AsRef<str>) -> Self {
        Self {
            entity,
            _attack: loc::<AttackResource>(attack_loc.as_ref())
                .expect(format!("Failed to parse attack resource location: {}", attack_loc.as_ref()).as_str()),
        }
    }
}

fn on_movement(
    event: On<InputMoveEvent>,
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
            match (event.toggle_sprint, controller.sprinting) {
                // We aren't sprinting, and don't want to sprint
                (false, false) => Box::new(Running),
                // We aren't sprinting, and want to start sprinting
                (false, true) => Box::new(Sprinting),
                // We are sprinting, and want to keep sprinting
                (true, false) => Box::new(Sprinting),
                // We are sprinting, and want to stop sprinting
                (true, true) => Box::new(Running),
            }
        }
    } else {
        Box::new(Idle)
    };

    let prev_state_id = tracker.state_type_id();
    let new_state_id = (*new_state).type_id();

    // If the character state has changed
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

// TODO: Convert jumping to use its own state
fn on_jump(
    event: On<InputJumpEvent>,
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

    if let PhysicsData::Kinematic {
        time_since_grounded,
        last_grounded_height,
        ..
    } = *physics {
        if time_since_grounded < COYOTE_TIME
            && position.as_vec3().y < last_grounded_height
            && position.as_vec3().y > last_grounded_height - COYOTE_TIME_HEIGHT_THRESHOLD
        {
            controller.intent.y = JUMP_VELOCITY;
        } else {
            info!("Cannot jump, player is not grounded!");
        }
    } else { panic!("Player assigned static physics data! This is a bug!") }
}

fn on_attack(
    event: On<InputAttackEvent>,
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
        // TODO: Move this into the character attack event
        let facing = {
            if let Some(aim_facing) = aim_facing.0 {
                *facing = aim_facing;
            }
            *facing
        };

        let attack_loc: ResourceLocation<AttackResource> = "player/basic_attack".parse().unwrap();

        commands.trigger(PlayerAttackEvent {
            entity: player_entity,
            facing,
            attack: attack_loc.clone(),
        });

        let Some(attack) = attack_registry.get_asset(&attack_loc) else {
            error!("Attack resource {} does not exist!", attack_loc);
            return;
        };

        let attack_state = Box::new(Attacking::new(&attack_loc, *attack.duration()));
        commands.trigger(TrySetStateEvent::new(player_entity, attack_state));
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