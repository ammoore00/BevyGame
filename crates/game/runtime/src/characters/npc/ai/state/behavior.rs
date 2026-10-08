use crate::characters::npc::ai::AiSystems;
use crate::characters::npc::ai::pathfinding::strategy::follow::FollowerState;
use crate::characters::npc::ai::state::AiState;
use crate::characters::npc::ai::state::transition::SetStateEvent;
use crate::debug::AiStateKind;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use common::WorldPosition;

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

#[derive(QueryData)]
struct AiComponents {
    entity: Entity,
    ai_state: &'static AiState,
    pos: &'static WorldPosition,
    follower_state: Option<&'static FollowerState>,
}

fn update_ai(
    npc_query: Query<AiComponents>,
    target_query: Query<&WorldPosition>,
    mut commands: Commands,
) {
    for components in npc_query {
        match components.ai_state.current {
            AiStateKind::Idle => {}
            AiStateKind::Wander => {}
            AiStateKind::Alert => {}
            AiStateKind::Attack => {
                let Some(follower_state) = components.follower_state else {
                    commands.trigger(SetStateEvent::new(components.entity, AiStateKind::Idle));
                    error!("Entity in attack state without follower data!");
                    continue;
                };

                let Some(target) = follower_state.target() else {
                    commands.trigger(SetStateEvent::new(components.entity, AiStateKind::Idle));
                    info!("Entity in attack state without target, setting to idle.");
                    continue;
                };

                let Ok(target_pos) = target_query.get(target) else {
                    commands.trigger(SetStateEvent::new(components.entity, AiStateKind::Idle));
                    error!("Target entity does not have a position in the world!");
                    continue;
                };

                let distance = components.pos.0.distance(*target_pos.0);
            }
        }
    }
}
