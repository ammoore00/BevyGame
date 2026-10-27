mod impls;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::characters::npc::ai::state::behavior::sense::SensedTargets;
use crate::characters::npc::ai::{AiSystems, NpcHome};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use getset::Getters;
use crate::characters::npc::ai::state::behavior::goal::impls::{AttackGoal, BackHomeGoal};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(impls::plugin);

    app.add_systems(Update, update_timers.in_set(AiSystems::PreUpdate));
    app.add_systems(Update, update_ai_goal.in_set(AiSystems::Calculate));
}

// TODO: Change this to pull from character data
pub fn goals_scene() -> impl Scene {
    let goal_list = GoalList {
        goals: vec![
            Arc::new(AttackGoal),
            Arc::new(BackHomeGoal),
        ],
    };
    
    bsn! {
        goal_list
        GoalState
    }
}

#[derive(SystemParam)]
pub struct GoalSystemData<'w, 's> {
    /// Entities that this entity has detected
    sensed_targets: Query<'w, 's, &'static SensedTargets>,

    homes: Query<'w, 's, &'static NpcHome>,
}

/// Information relevant to the current goal decisions of the AI
pub struct GoalData<'a> {
    sensed_targets: &'a SensedTargets,

    home: Option<NpcHome>,
}
impl<'a> GoalData<'a> {
    fn try_from_system_data(entity: Entity, goal_data: &'a GoalSystemData<'_, '_>) -> Result<Self, GoalDataError> {
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
    #[error("No data on entity {0} for entry '{1}'")]
    NoData(Entity, String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GoalId(String);
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

pub trait Goal: Debug + Send + Sync + 'static {
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
impl PartialEq for dyn Goal {
    fn eq(&self, other: &dyn Goal) -> bool {
        self.id() == other.id()
    }
}
impl Eq for dyn Goal {}
impl Hash for dyn Goal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}

#[derive(Component, Debug, Clone, Getters)]
pub struct GoalList {
    #[getset(get = "pub")]
    goals: Vec<Arc<dyn Goal>>,
}
impl Display for GoalList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.goals)
    }
}

#[derive(Component, Debug, Clone, Default, Getters)]
pub struct GoalState {
    #[getset(get = "pub")]
    active_goal: Option<Arc<dyn Goal>>,
    #[getset(get = "pub")]
    time_in_goal: Duration,
    #[getset(get = "pub")]
    scores: HashMap<Arc<dyn Goal>, Option<u32>>,
}
impl GoalState {
    fn populate_scores(&mut self, goals: &[Arc<dyn Goal>], data: &GoalData) {
        self.scores.clear();
        goals.iter().for_each(|goal| {
            self.scores.insert(goal.clone(), goal.get_score(data));
        });
        
        let best_goal = goals
            .iter()
            .filter_map(|goal| goal.get_score(data).map(|score| (goal, score)))
            .max_by_key(|(_, score)| *score)
            .map(|(goal, _)| goal.clone());
        
        self.active_goal = best_goal;
    }
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
        let goal_data = match GoalData::try_from_system_data(entity, &goal_data) {
            Ok(goal_data) => goal_data,
            Err(err) => {
                error!("{}", err);
                continue;
            }
        };
        
        state.populate_scores(&goals.goals, &goal_data);
    }
}
