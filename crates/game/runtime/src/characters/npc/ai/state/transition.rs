use std::time::Duration;
use crate::characters::npc::ai::pathfinding::strategy::NoPathfinding;
use crate::characters::npc::ai::pathfinding::strategy::follow::Following;
use crate::characters::npc::ai::pathfinding::strategy::wander::Wandering;
use crate::characters::npc::ai::state::{AiState, AiStateKind};
use crate::prelude::{GainedTarget, Player};
use bevy::prelude::*;
use crate::characters::npc::ai::NpcHome;
use crate::characters::npc::ai::pathfinding::strategy::position::MoveToPosEvent;

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
            state.time_in_state = Duration::ZERO;
        }
        Err(err) => error!("Error getting AiState: {err:?}"),
    }
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
struct StateTransitionEvent {
    entity: Entity,
    new_state: AiStateKind,
    _prev_state: AiStateKind,
}

// TODO: Replace temporary logic with real data-driven logic
fn on_state_transition(
    event: On<StateTransitionEvent>,
    home_query: Query<Option<&NpcHome>>,
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
            commands.entity(event.entity).apply_scene(bsn! { @Following });
            // TODO: Proper detection
            commands.trigger(GainedTarget::new(event.entity, player.entity()));
        }
        AiStateKind::BackHome => {
            let Some(home) = home_query.get(event.entity).unwrap() else {
                commands.trigger(SetStateEvent::new(event.entity, AiStateKind::Idle));
                error!("Back home state triggered for character without a home!");
                return;
            };
            commands.trigger(MoveToPosEvent::new(event.entity, home.pos))
        }
    }
}
