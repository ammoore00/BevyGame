mod goal;
pub mod sense;

use crate::characters::attack::{AttackSuccessEvent, TryAttackEvent};
use crate::characters::npc::ai::pathfinding::TargetGoal;
use crate::characters::npc::ai::pathfinding::strategy::follow::FollowerState;
use crate::characters::npc::ai::{AiSystems, NpcHome};
use crate::prelude::*;
use assets::resource::characters::AiBehaviorParams;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use common::{Facing, WorldPosition};
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((goal::plugin, sense::plugin));
    
    app.add_systems(
        Update,
        (
            update_timers.in_set(AiSystems::PreUpdate),
            set_ai_state.in_set(AiSystems::Execute),
        ),
    );

    app.add_observer(on_successful_attack);
}

fn update_timers(npc_query: Query<&mut AiState>, time: Res<Time>) {
    for mut ai_state in npc_query {
        ai_state.time_in_state += time.delta();

        if let Some(time_since_last_attack) = ai_state.time_since_last_attack.as_mut() {
            *time_since_last_attack += time.delta();
        }
    }
}

#[derive(QueryData)]
struct AiComponents {
    entity: Entity,
    ai_state: &'static AiState,
    ai_behavior: &'static AiBehaviorParams,

    pos: &'static WorldPosition,
    facing: &'static Facing,

    target_goal: Option<&'static TargetGoal>,
    follower_state: Option<&'static FollowerState>,

    home: Option<&'static NpcHome>,
}

const ATTACK_RANGE: f32 = 1.0;
const ATTACK_DELAY: f32 = 1.0;
const FOLLOW_RANGE: f32 = 5.0;

const MAX_IDLE_TIME: Duration = Duration::from_millis(5000);
const IDLE_BACK_HOME_DIST: f32 = 1.0;

fn set_ai_state(
    npc_query: Query<AiComponents>,
    player_query: Single<(Entity, &WorldPosition), With<Player>>,
    target_query: Query<&WorldPosition>,
    mut commands: Commands,
) {
    let (_, player_pos) = player_query.into_inner();

    for components in npc_query {
        let mut check_for_aggro = true;

        if let Some(leash_distance) = components.ai_behavior.leash_distance()
            && let Some(home) = components.home
        {
            let distance = components.pos.0.distance(home.pos.0);

            if distance > leash_distance {
                commands.trigger(SetStateEvent::new(components.entity, AiStateKind::BackHome));
                continue;
            }
        }

        match components.ai_state.current {
            AiStateKind::Idle => {
                if components.ai_state.time_in_state > MAX_IDLE_TIME {
                    if let Some(home) = components.home {
                        let distance = components.pos.0.distance(home.pos.0);
                        if distance > IDLE_BACK_HOME_DIST {
                            commands.trigger(SetStateEvent::new(
                                components.entity,
                                AiStateKind::BackHome,
                            ));
                        }
                    }
                }
            }
            AiStateKind::Wander => {}
            AiStateKind::Alert => {}
            AiStateKind::Attack => {
                let time_since_last_attack = components.ai_state.time_since_last_attack;
                if let Some(time) = time_since_last_attack
                    && time.as_secs_f32() < ATTACK_DELAY
                {
                    continue;
                }

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
            AiStateKind::BackHome => {
                let Some(home) = components.home else {
                    commands.trigger(SetStateEvent::new(components.entity, AiStateKind::Idle));
                    error!("Back home state behavior called for character without a home!");
                    continue;
                };

                let threshold_dist =
                    if let Some(TargetGoal::Position(target_pos)) = components.target_goal {
                        target_pos.threshold_dist()
                    } else {
                        // TODO: This currently takes one frame from BackHome being set to there being an actual target.
                        //  Theoretically there is nothing wrong with that, but it should probably be investigated.
                        continue;
                    };

                let distance = components.pos.0.distance(home.pos.0);
                if distance < threshold_dist {
                    commands.trigger(SetStateEvent::new(components.entity, AiStateKind::Idle));
                }

                // Do not re-aggro until we get closer to home
                if let Some(leash_distance) = components.ai_behavior.leash_distance()
                    && distance > leash_distance / 2.0
                {
                    check_for_aggro = false;
                }
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

fn on_successful_attack(event: On<AttackSuccessEvent>, mut ai_query: Query<&mut AiState>) {
    let Ok(mut ai_state) = ai_query.get_mut(event.entity) else {
        return;
    };

    ai_state.last_attack = Some(event.attack().clone());
    ai_state.time_since_last_attack = Some(Duration::ZERO);
}
