use crate::characters::npc::ai::state::behavior::goal::{Goal, GoalData, GoalId};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {}

#[derive(Debug)]
struct AttackGoal {}
impl Goal for AttackGoal {
    fn id(&self) -> GoalId {
        "Attack".into()
    }

    fn get_score(&self, data: &GoalData) -> Option<u32> {
        todo!()
    }
}
