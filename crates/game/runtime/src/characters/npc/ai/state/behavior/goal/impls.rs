use crate::characters::npc::ai::state::behavior::goal::{Goal, GoalData, GoalId};
use bevy::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

#[derive(Debug)]
struct AttackGoal;
impl Goal for AttackGoal {
    fn id(&self) -> GoalId {
        "Attack".into()
    }

    fn get_score(&self, data: &GoalData) -> Option<u32> {
        // TODO: Add support for properly choosing targets instead of the first detected
        data.sensed_targets.targets().iter().next().map(|_| 100)
    }
}

#[derive(Debug)]
struct BackHome;
impl Goal for BackHome {
    fn id(&self) -> GoalId {
        "BackHome".into()
    }

    fn get_score(&self, data: &GoalData) -> Option<u32> {
        data.home.map(|_| 50)
    }
}
