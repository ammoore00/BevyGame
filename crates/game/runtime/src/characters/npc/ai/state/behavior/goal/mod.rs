mod impls;

use std::fmt::{Display, Formatter, Write};
use crate::characters::npc::ai::state::behavior::sense::SensedTargets;
use crate::characters::npc::ai::{AiSystems, NpcHome};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(impls::plugin);

    app.add_systems(Update, update_timers.in_set(AiSystems::PreUpdate));
    app.add_systems(Update, update_ai_goal.in_set(AiSystems::Calculate));
}

#[derive(SystemParam)]
struct GoalSystemData<'w, 's> {
    /// Entities that this entity has detected
    sensed_targets: Query<'w, 's, &'static SensedTargets>,

    homes: Query<'w, 's, &'static NpcHome>,
}

/// Information relevant to the current goal decisions of the AI
struct GoalData<'a> {
    sensed_targets: &'a SensedTargets,

    home: Option<NpcHome>,
}
impl<'a> GoalData<'a> {
    fn from_system_data(entity: Entity, goal_data: &'a GoalSystemData<'_, '_>) -> Result<Self, GoalDataError> {
        let sensed_targets = goal_data
            .sensed_targets
            .get(entity)
            .map_err(|_| GoalDataError::NoData(entity, "sensed_targets".to_string()))?;

        let home = goal_data.homes.get(entity).cloned().ok();

        Ok(Self {
            sensed_targets,
            home,
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
enum GoalDataError {
    #[error("No data for entity {0} for entry {1}")]
    NoData(Entity, String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GoalId(String);
impl From<&str> for GoalId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
impl From<String> for GoalId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

trait Goal: Debug + Send + Sync + 'static {
    fn id(&self) -> GoalId;

    /// Based on the current state of the AI, how important is this goal
    ///
    /// Returns None if the goal is not applicable
    fn get_score(&self, data: &GoalData) -> Option<u32>;
}
impl Display for dyn Goal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id().0)
    }
}

#[derive(Component, Debug)]
struct GoalList {
    goals: Vec<Arc<dyn Goal>>,
}
impl GoalList {
    fn get_best_goal(&self, data: &GoalData) -> Option<Arc<dyn Goal>> {
        self.goals
            .iter()
            .filter_map(|goal| goal.get_score(data).map(|score| (goal, score)))
            .max_by_key(|(_, score)| *score)
            .map(|(goal, _)| goal.clone())
    }
}

#[derive(Component, Debug)]
struct GoalState {
    active_goal: Option<Arc<dyn Goal>>,
    time_in_goal: Duration,
}

fn update_timers(mut state_query: Query<&mut GoalState>, time: Res<Time>) {
    for mut state in state_query.iter_mut() {
        state.time_in_goal += time.delta();
    }
}

fn update_ai_goal(
    state_query: Query<(Entity, &mut GoalState, &GoalList)>,
    goal_data: GoalSystemData,
) {
    for (entity, mut state, goals) in state_query {
        let goal_data = match GoalData::from_system_data(entity, &goal_data) {
            Ok(goal_data) => goal_data,
            Err(err) => {
                error!("{}", err);
                continue;
            }
        };

        let best_goal = goals.get_best_goal(&goal_data);

        if let Some(best_goal) = best_goal {
            state.active_goal = Some(best_goal.clone());
        }
    }
}
