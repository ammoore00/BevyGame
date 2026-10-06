use crate::characters::npc::ai::pathfinding::strategy::NoPathfinding;
use crate::characters::npc::ai::pathfinding::strategy::follow::Following;
use crate::characters::npc::ai::pathfinding::strategy::wander::Wandering;
use crate::characters::npc::ai::state::{AiState, AiStateKind};
use crate::debug::{GainedTarget, Player};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_set_state);
    app.add_observer(on_state_transition);
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
pub struct SetStateEvent {
    entity: Entity,
    state: AiStateKind,
}

fn on_set_state(event: On<SetStateEvent>, mut query: Query<&mut AiState>, mut commands: Commands) {
    match query.get_mut(event.entity) {
        Ok(mut state) => {
            state.current = event.state;
            commands.trigger(StateTransitionEvent::new(
                event.entity,
                state.current,
                state.prev,
            ));
        }
        Err(err) => error!("Error getting AiState: {err:?}"),
    }
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
struct StateTransitionEvent {
    entity: Entity,
    new_state: AiStateKind,
    prev_state: AiStateKind,
}

// TODO: Replace temporary logic with real data-driven logic
fn on_state_transition(
    event: On<StateTransitionEvent>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    match event.new_state {
        AiStateKind::Idle => {
            commands.entity(event.entity).insert(NoPathfinding);
        }
        AiStateKind::Wander => {
            commands.entity(event.entity).insert(Wandering);
        }
        AiStateKind::Alert => {
            commands.entity(event.entity).insert(Wandering);
        }
        AiStateKind::Attack => {
            commands.entity(event.entity).apply_scene(bsn![@Following]);
            // TODO: Proper detection
            commands.trigger(GainedTarget::new(event.entity, player.entity()));
        }
    }
}
