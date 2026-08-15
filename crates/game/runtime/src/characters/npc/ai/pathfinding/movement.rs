use bevy::prelude::*;
use assets::action_states::{ActionState, ActionStateCapabilities, Idle, Running, Walking};
use common::WorldPosition;
use physics::MovementController;
use crate::characters::npc::ai::AiSystems;
use crate::characters::npc::ai::pathfinding::pathfinder::{Pathfinder, Waypoints};
use crate::characters::state::{ActionStateTracker, TrySetStateEvent};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_movement_intent, update_movement_state)
            .chain()
            .in_set(AiSystems::Execute),
    );
}

/// Update the movement controller based on what the pathfinder wants
fn update_movement_intent(
    pathfinder_query: Query<(&mut MovementController, &WorldPosition, Option<&Waypoints>), With<Pathfinder>>,
) {
    for (mut controller, pos, waypoints) in pathfinder_query {
        if let Some(waypoints) = waypoints {
            // TODO: Clean up unwrap here - this should theoretically never fail but safety does not hurt
            let delta = **waypoints.next_position.as_ref().unwrap() - *pos.0;
            let delta = delta * Vec3::new(1., 0., 1.);

            if delta.length() < 0.01 {
                controller.intent = Vec3::ZERO;
            } else {
                controller.intent = delta.normalize();
            }
        } else {
            controller.intent = Vec3::ZERO;
        }
    }
}

/// Update the action state based on the movement intent
// TODO: Remove the direct world access here
fn update_movement_state(world: &mut World) {
    let mut npc_query = world.query_filtered::<Entity, (
        With<MovementController>,
        With<ActionStateTracker>,
        With<Pathfinder>,
        With<ActionStateCapabilities>,
    )>();
    let npc_query: Vec<_> = npc_query.iter(world).collect();

    for entity in npc_query {
        let controller = world.get::<MovementController>(entity).unwrap();

        let new_state: Box<dyn ActionState> = if controller.intent.length() > 0.7 {
            Box::new(Running)
        } else if controller.intent.length() > 0.01 {
            Box::new(Walking)
        } else {
            Box::new(Idle)
        };

        let state = world.get::<ActionStateTracker>(entity).unwrap();
        if (*new_state).type_id() == state.state_type_id() {
            continue;
        }

        world.trigger(TrySetStateEvent::new(entity, new_state));
    }
}