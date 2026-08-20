use crate::characters::npc::ai::pathfinding::pathfinder::{
    PathfinderState, TARGET_REACHED_THRESHOLD,
};
use crate::characters::npc::ai::pathfinding::strategy::{
    PathfindStrategy, PathfindStrategyRegistry, ReflectPathfindStrategy,
};
use crate::characters::npc::ai::pathfinding::{PathfinderData, PathfinderSystems};
use crate::debug::TileNavMap;
use crate::level::LEVEL_LOADED;
use bevy::prelude::*;
use common::WorldPosition;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_follower_state.in_set(PathfinderSystems::Update),
            follow_dispatch.in_set(PathfinderSystems::Dispatch),
        ),
    );

    app.register_pathfind_strategy::<Following>();

    app.add_observer(on_gained_target);
    app.add_observer(on_lost_target);

    app.add_observer(on_following_added);
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

#[derive(Component, Default, Debug, Clone, PartialEq, Eq)]
struct FollowerState {
    /// The current target to follow
    target: Option<Entity>,

    /// The timer for re-pathing
    re_path_timer: Option<Timer>,
    /// Flag to indicate that the follower should re-path
    re_path_flag: bool,
}
impl FollowerState {
    fn should_re_path(&self) -> bool {
        self.re_path_flag
            || self
                .re_path_timer
                .as_ref()
                .is_some_and(|timer| timer.is_finished())
    }

    fn on_re_path(&mut self) {
        // If this was a manual re-path, reset the timer
        // Otherwise, the timer will reset itself
        if self.re_path_flag
            && let Some(timer) = self.re_path_timer.as_mut()
        {
            timer.reset()
        }
        self.re_path_flag = false;
    }
}

fn on_following_added(
    event: On<Add, Following>,
    query: Query<(), With<FollowerData>>,
    mut commands: Commands,
) {
    if query.get(event.entity).is_err() {
        commands.entity(event.entity).remove::<Following>();
        error!("Cannot set entity without FollowerData to Following!");
    }
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
    re_path_time: Duration,
}
impl Default for FollowerData {
    fn default() -> Self {
        Self {
            stop_distance: TARGET_REACHED_THRESHOLD,
            re_path_threshold: RE_PATH_THRESHOLD,
            re_path_time: RE_PATH_TIMER,
        }
    }
}

fn update_follower_state(pathfinder_query: Query<(&FollowerData, &mut FollowerState)>) {
    for (follower_data, mut follower_state) in pathfinder_query {
        // If the timer hasn't been initialized, initialize it
        // This is done because the data for how often to re-path is stored in a different component so cannot be retrieved
        //  during construction
        if follower_state.re_path_timer.is_none() {
            follower_state.re_path_timer = Some(Timer::from_seconds(
                follower_data.re_path_time.as_secs_f32(),
                TimerMode::Repeating,
            ));
            // Trigger an immediate re-path since we won't have any path yet
            follower_state.re_path_flag = true;
        }
    }
}

fn follow_dispatch(
    pathfinder_query: Query<(PathfinderData, &FollowerData, &mut FollowerState), With<Following>>,
    nav_map_query: Query<&TileNavMap>,
    mut commands: Commands,
) {
    let nav_map = nav_map_query.single();
    let Ok(nav_map) = nav_map else {
        error!("Failed to get nav map!: {:?}", nav_map.err().unwrap());
        return;
    };

    for (pathfinder_data, follower_data, mut follower_state) in pathfinder_query {
        if pathfinder_data.pathfinder.state() != PathfinderState::Dispatch {
            continue;
        }

        if !follower_state.should_re_path() {
            continue;
        }

        follower_state.on_re_path();

        todo!()
    }
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GainedTarget {
    pub entity: Entity,
    pub target: Entity,
}

fn on_gained_target(
    event: On<GainedTarget>,
    follower_query: Query<
        (PathfinderData, &mut FollowerState),
        (With<Following>, With<FollowerData>),
    >,
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
    follower_query: Query<
        (PathfinderData, &mut FollowerState),
        (With<Following>, With<FollowerData>),
    >,
) {
    todo!()
}
