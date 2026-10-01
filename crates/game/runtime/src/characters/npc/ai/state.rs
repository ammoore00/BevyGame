use crate::characters::npc::ai::AiSystems;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_prev_state.in_set(AiSystems::Cleanup));

    app.add_observer(on_set_state);
}

pub fn state_scene() -> impl Scene {
    bsn![AiState]
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AiState {
    current: AiStateKind,
    prev: AiStateKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AiStateKind {
    #[default]
    Idle,
    Wander,
    Alert,
    Attack,
}

fn update_prev_state(query: Query<&mut AiState>) {
    for mut ai_state in query {
        if ai_state.prev != ai_state.current {
            ai_state.prev = ai_state.current;
        }
    }
}

#[derive(EntityEvent, Debug, Clone)]
pub struct SetStateEvent {
    entity: Entity,
    state: AiStateKind,
}

fn on_set_state(
    event: On<SetStateEvent>,
    mut query: Query<&mut AiState>,
) {
    match query.get_mut(event.entity) {
        Ok(mut state) => state.current = event.state,
        Err(err) => error!("Error getting AiState: {err:?}"),
    }
}
