use crate::characters::attack::TryAttackEvent;
use crate::characters::npc::ai::AiSystems;
use crate::characters::npc::ai::pathfinding::strategy::follow::FollowerState;
use crate::prelude::*;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use common::{Facing, WorldPosition};

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
    facing: &'static Facing,
    follower_state: Option<&'static FollowerState>,
}

const ATTACK_RANGE: f32 = 1.0;
const FOLLOW_RANGE: f32 = 5.0;

fn update_ai(
    npc_query: Query<AiComponents>,
    player_query: Single<(Entity, &WorldPosition), With<Player>>,
    target_query: Query<&WorldPosition>,
    mut commands: Commands,
) {
    let (_, player_pos) = player_query.into_inner();

    for components in npc_query {
        let mut check_for_aggro = true;

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

                if distance < ATTACK_RANGE {
                    commands.trigger(TryAttackEvent::new(
                        components.entity,
                        *components.facing,
                        "test/basic_attack".parse().unwrap(),
                    ));
                } else if distance > FOLLOW_RANGE {
                    commands.trigger(SetStateEvent::new(components.entity, AiStateKind::Idle));
                }

                check_for_aggro = false;
            }
        }

        if check_for_aggro {
            let distance = components.pos.0.distance(*player_pos.0);

            if distance <= FOLLOW_RANGE {
                commands.trigger(SetStateEvent::new(components.entity, AiStateKind::Attack));
            }
        }
    }
}
