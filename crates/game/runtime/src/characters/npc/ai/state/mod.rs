use std::time::Duration;
use crate::characters::npc::ai::AiSystems;
use bevy::prelude::*;

pub mod transition;
mod behavior;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((behavior::plugin, transition::plugin));
    
    app.add_systems(Update, update_prev_state.in_set(AiSystems::Cleanup));
}

pub fn state_scene() -> impl Scene {
    bsn![AiState]
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AiState {
    current: AiStateKind,
    prev: AiStateKind,
    time_in_state: Duration,
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