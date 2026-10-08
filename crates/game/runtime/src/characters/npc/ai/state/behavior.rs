use crate::characters::npc::ai::AiSystems;
use crate::characters::npc::ai::state::AiState;
use crate::debug::AiStateKind;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            calculate_intent.in_set(AiSystems::Calculate),
            update_ai.in_set(AiSystems::Execute),
        ),
    );
}

fn calculate_intent(npc_query: Query<&mut AiState>, time: Res<Time>) {
    for mut ai_state in npc_query {
        ai_state.time_in_state += time.delta();
    }
}

fn update_ai(npc_query: Query<(Entity, &AiState)>, mut commands: Commands) {
    for (entity, ai_state) in npc_query {
        match ai_state.current {
            AiStateKind::Idle => {}
            AiStateKind::Wander => {}
            AiStateKind::Alert => {}
            AiStateKind::Attack => {}
        }
    }
}
