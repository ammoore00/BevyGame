use crate::characters::npc::ai::pathfinding::PathfinderData;
use crate::characters::npc::ai::pathfinding::pathfinder::TARGET_REACHED_THRESHOLD;
use crate::characters::npc::ai::pathfinding::strategy::{
    PathfindStrategy, PathfindStrategyRegistry, ReflectPathfindStrategy,
};
use crate::level::LEVEL_LOADED;
use bevy::prelude::*;
use common::WorldPosition;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.register_pathfind_strategy::<Following>();

    app.add_observer(on_gained_target);
    app.add_observer(on_lost_target);

    app.add_observer(on_following_removed.run_if(in_state(LEVEL_LOADED)));
}

#[derive(SceneComponent, Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
#[reflect(PathfindStrategy)]
#[scene(FollowerProps)]
pub struct Following;
impl PathfindStrategy for Following {}
impl Following {
    pub fn scene(props: FollowerProps) -> impl Scene {
        bsn! [
            Following
            FollowerState {
                target: {props.target},
            }
        ]
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FollowerProps {
    target: Option<Entity>,
}

#[derive(Component, Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct FollowerState {
    target: Option<Entity>,
}

fn on_following_removed(event: On<Remove, Following>, mut commands: Commands) {
    commands.entity(event.entity).remove::<FollowerState>();
}

const RE_PATH_THRESHOLD: f32 = 1.0;
const RE_PATH_TIMER: Duration = Duration::from_millis(500);

#[derive(Component, Debug, Clone)]
pub struct FollowerData {
    /// The distance from the target to be considered close enough to stop
    stop_distance: f32,
    /// The distance the target must move before the follower re-paths
    re_path_threshold: f32,
    /// The timer for re-pathing
    re_path_timer: Timer,
}
impl Default for FollowerData {
    fn default() -> Self {
        Self {
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
    follower_query: Query<(PathfinderData, &FollowerData, &mut FollowerState), With<Following>>,
    target_query: Query<&WorldPosition>,
) {
    todo!()
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LostTarget {
    pub entity: Entity,
}

fn on_lost_target(
    event: On<LostTarget>,
    follower_query: Query<(PathfinderData, &FollowerData, &mut FollowerState), With<Following>>,
) {
    todo!()
}
