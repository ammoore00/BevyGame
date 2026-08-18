use crate::characters::npc::ai::AiState;
use crate::characters::npc::ai::pathfinding::pathfinder::{Pathfinder, TARGET_REACHED_THRESHOLD};
use bevy::prelude::*;
use common::WorldPosition;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_gained_target);
    app.add_observer(on_lost_target);
}

const RE_PATH_THRESHOLD: f32 = 1.0;
const RE_PATH_TIMER: Duration = Duration::from_millis(500);

#[derive(Component, Debug, Clone)]
pub struct TargetFollower {
    /// The entity to follow
    target: Entity,
    /// The distance from the target to be considered close enough to stop
    stop_distance: f32,
    /// The distance the target must move before the follower re-paths
    re_path_threshold: f32,
    /// The timer for re-pathing
    re_path_timer: Timer,
}
impl TargetFollower {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            stop_distance: TARGET_REACHED_THRESHOLD,
            re_path_threshold: RE_PATH_THRESHOLD,
            re_path_timer: Timer::from_seconds(RE_PATH_TIMER.as_secs_f32(), TimerMode::Repeating),
        }
    }
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GainedTarget {
    pub entity: Entity,
    pub target: Entity,
}

fn on_gained_target(
    event: On<GainedTarget>,
    follower_query: Query<(&mut AiState, Option<&mut TargetFollower>), With<Pathfinder>>,
    target_query: Query<&WorldPosition>,
) {
    todo!()
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LostTarget {
    pub entity: Entity,
}

fn on_lost_target(
    event: On<GainedTarget>,
    follower_query: Query<(&mut AiState, &mut TargetFollower), With<Pathfinder>>,
) {
    todo!()
}
