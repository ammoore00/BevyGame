use crate::characters::npc::ai::AiSystems;
use crate::characters::npc::ai::pathfinding::pathfinder::{Pathfinder, Waypoints};
use crate::characters::state::{ActionStateTracker, TrySetStateEvent};
use assets::action_states::{ActionState, ActionStateCapabilities, Idle, Running, Walking};
use bevy::prelude::*;
use common::WorldPosition;
use physics::MovementController;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_movement_intent, update_movement_state)
            .chain()
            .in_set(AiSystems::Execute),
    );

    app.add_observer(on_movement_attempt);
}

#[derive(Component, Debug, Clone, Default)]
pub(super) struct AiMovementIntent(Vec3);

/// Update the movement controller based on what the pathfinder wants
fn update_movement_intent(
    pathfinder_query: Query<
        (&mut AiMovementIntent, &WorldPosition, Option<&Waypoints>),
        With<Pathfinder>,
    >,
) {
    for (mut intent, pos, waypoints) in pathfinder_query {
        if let Some(waypoints) = waypoints {
            let Some(delta) = *waypoints.next_position() else {
                intent.0 = Vec3::ZERO;
                error!("No next position found for pathfinder");
                continue;
            };

            let delta = delta - *pos.0;
            let delta = delta * Vec3::from([1., 0., 1.]);

            if delta.length() < 0.01 {
                intent.0 = Vec3::ZERO;
            } else {
                intent.0 = delta.normalize();
            }
        } else {
            intent.0 = Vec3::ZERO;
        }
    }
}

/// Update the action state based on the movement intent
fn update_movement_state(
    npc_query: Query<
        (Entity, &AiMovementIntent, &ActionStateTracker),
        (With<Pathfinder>, With<ActionStateCapabilities>),
    >,
    mut commands: Commands,
) {
    for (entity, intent, state) in &npc_query {
        let new_state: Box<dyn ActionState> = if intent.0.length() > 0.7 {
            Box::new(Running)
        } else if intent.0.length() > 0.01 {
            Box::new(Walking)
        } else {
            Box::new(Idle)
        };

        if (*new_state).type_id() == state.state_type_id() {
            // If we didn't have to change states, then just trigger the event to update movement values if they changed
            commands.trigger(MovementAttemptEvent::new(entity, true));
            continue;
        }

        commands.trigger(TrySetStateEvent::new(entity, new_state).with_callback(
            |entity, mut commands: Commands, result: Result<_, _>| {
                commands.trigger(MovementAttemptEvent::new(entity, result.is_ok()))
            },
        ));
    }
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, Eq, Hash, derive_new::new)]
struct MovementAttemptEvent {
    entity: Entity,
    success: bool,
}

fn on_movement_attempt(
    event: On<MovementAttemptEvent>,
    mut query: Query<(&AiMovementIntent, &mut MovementController)>,
) {
    let (intent, mut controller) = match query.get_mut(event.entity) {
        Ok((intent, controller)) => (intent, controller),
        Err(err) => {
            error!("Failed to get movement intent and controller for entity: {err:?}");
            return;
        }
    };

    if event.success {
        controller.intent = intent.0;
    } else {
        controller.intent = Vec3::ZERO;
    }
}
