use crate::characters::npc::ai::AiSystems;
use crate::characters::npc::ai::state::AiState;
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

fn calculate_intent(ai_state_query: Query<&AiState>) {}

fn update_ai(ai_state_query: Query<(Entity, &AiState)>, mut commands: Commands) {
    
}
