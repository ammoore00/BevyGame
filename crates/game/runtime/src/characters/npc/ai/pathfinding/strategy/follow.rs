use crate::characters::npc::ai::pathfinding::pathfinder::{
    CancelPathing, PathfindRequest, PathfinderState, TARGET_REACHED_THRESHOLD,
};
#[cfg(test)]
use crate::characters::npc::ai::pathfinding::strategy::follow::test::GainLoseTargetError;
use crate::characters::npc::ai::pathfinding::strategy::{
    PathfindStrategy, PathfindStrategyRegistry, ReflectPathfindStrategy,
};
use crate::characters::npc::ai::pathfinding::{PathfinderData, PathfinderSystems};
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
    }

    /// Call for a re-path the next time the pathfinder is checked
    fn trigger_re_path(&mut self) {
        if let Some(timer) = self.re_path_timer.as_mut() {
            timer.reset()
        }
        self.re_path_flag = true;
    }

    /// Clear the flag once a re-path has been performed
    fn on_re_path(&mut self) {
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

        if follower_state.re_path_timer.as_ref().unwrap().is_finished() {
            follower_state.trigger_re_path();
        }
    }
}

fn follow_dispatch(
    pathfinder_query: Query<
        (PathfinderData, &mut FollowerState),
        (With<Following>, With<FollowerData>),
    >,
    target_query: Query<&WorldPosition>,
    mut commands: Commands,
) {
    for (pathfinder_data, mut follower_state) in pathfinder_query {
        if pathfinder_data.pathfinder.state() != PathfinderState::Dispatch {
            continue;
        }

        if !follower_state.should_re_path() {
            continue;
        }

        follower_state.on_re_path();

        let Some(target) = follower_state.target else {
            warn!("Re-path triggered for follower without a target!");
            continue;
        };

        let Ok(target_pos) = target_query.get(target) else {
            error!("Failed to get target's position!");
            continue;
        };

        let request = PathfindRequest::new(
            pathfinder_data.pos.0,
            target_pos.0,
            pathfinder_data.clearance(),
        );
        commands.entity(pathfinder_data.entity).insert(request);

        info!("NPC started searching");
    }
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GainedTarget {
    pub entity: Entity,
    pub target: Entity,
}

fn on_gained_target(
    event: On<GainedTarget>,
    mut follower_query: Query<&mut FollowerState, (With<Following>, With<FollowerData>)>,
    #[cfg(test)] mut commands: Commands,
) {
    let Ok(mut follower_state) = follower_query.get_mut(event.entity) else {
        let err =
            "Cannot gain target without appropriate follower data and while in follower state!";

        #[cfg(test)]
        commands
            .entity(event.entity)
            .insert(GainLoseTargetError(err.to_string()));

        error!(err);
        return;
    };

    if follower_state.target != Some(event.target) {
        follower_state.target = Some(event.target);
        follower_state.trigger_re_path();
    }
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LostTarget {
    pub entity: Entity,
}

fn on_lost_target(
    event: On<LostTarget>,
    mut follower_query: Query<&mut FollowerState, With<Following>>,
    mut commands: Commands,
) {
    let Ok(mut follower_state) = follower_query.get_mut(event.entity) else {
        let err = "Cannot lose target if not in following state!";

        #[cfg(test)]
        commands
            .entity(event.entity)
            .insert(GainLoseTargetError(err.to_string()));

        error!(err);
        return;
    };

    follower_state.target = None;
    commands.entity(event.entity).trigger(CancelPathing);
}

#[cfg(test)]
mod test {
    use super::*;

    mod follow_state {
        use super::*;
    }

    pub use target_events::GainLoseTargetError;

    mod target_events {
        use super::*;
        use crate::characters::npc::ai::pathfinding::pathfinder::test_on_cancel_pathing;
        use bevy::scene::ScenePlugin;

        #[derive(Component, Debug, Clone, thiserror::Error)]
        #[error("{}", .0)]
        pub struct GainLoseTargetError(pub String);

        struct TargetTestFixture {
            app: App,
            follower: Entity,
            existing_target: Option<Entity>,
        }

        fn setup_targets(has_target: bool) -> TargetTestFixture {
            let mut app = App::new();
            app.add_plugins(AssetPlugin::default());
            app.add_plugins(ScenePlugin);

            app.add_observer(on_following_added);
            app.add_observer(on_following_removed);

            app.add_observer(on_gained_target);
            app.add_observer(on_lost_target);

            app.add_observer(test_on_cancel_pathing);

            let target = if has_target {
                Some(app.world_mut().spawn_empty().id())
            } else {
                None
            };

            let follower = app
                .world_mut()
                .spawn_scene(bsn![
                    FollowerData
                    @Following {
                        @target
                    }
                ])
                .unwrap()
                .id();

            TargetTestFixture {
                app,
                follower,
                existing_target: target,
            }
        }

        mod gain_target {
            use super::*;

            #[test]
            fn gain_target() {
                // GIVEN
                // An entity in the following state without a target
                let TargetTestFixture {
                    mut app, follower, ..
                } = setup_targets(false);

                // WHEN
                // We give it a target
                let expected_target = app.world_mut().spawn_empty().id();
                app.world_mut().trigger(GainedTarget {
                    entity: follower,
                    target: expected_target,
                });

                app.update();

                // THEN
                // It should update its target as expected
                let mut query = app.world_mut().query::<&FollowerState>();
                let follower_state = query.get(app.world(), follower);

                assert!(follower_state.is_ok(), "Cannot get follower state!");

                let current_target = follower_state.unwrap().target;

                assert!(current_target.is_some(), "Entity has no target!");
                assert_eq!(
                    expected_target,
                    current_target.unwrap(),
                    "Target did not match expected!"
                );
            }

            #[test]
            fn gain_target_change_target() {
                // GIVEN
                // An entity in the following state that already has a target
                let TargetTestFixture {
                    mut app, follower, ..
                } = setup_targets(true);

                // WHEN
                // We give it a new target
                let new_target = app.world_mut().spawn_empty().id();
                app.world_mut().trigger(GainedTarget {
                    entity: follower,
                    target: new_target,
                });

                app.update();

                // THEN
                // It should update its target to the new one and trigger a re-path
                let mut query = app.world_mut().query::<&FollowerState>();
                let follower_state = query.get(app.world(), follower);

                assert!(follower_state.is_ok(), "Cannot get follower state!");

                let follower_state = follower_state.unwrap();
                let current_target = follower_state.target;

                assert!(current_target.is_some(), "Entity has no target!");
                assert_eq!(
                    new_target,
                    current_target.unwrap(),
                    "Target did not match expected!"
                );

                assert!(follower_state.should_re_path(), "Re-path not triggered!");
            }

            #[test]
            fn gain_target_same_target() {
                // GIVEN
                // An entity in the following state that already has a target
                let TargetTestFixture {
                    mut app,
                    follower,
                    existing_target,
                } = setup_targets(true);

                // WHEN
                // We try to assign it the same target again
                app.world_mut().trigger(GainedTarget {
                    entity: follower,
                    target: existing_target.unwrap(),
                });

                app.update();

                // THEN
                // Nothing should happen
                let mut query = app.world_mut().query::<&FollowerState>();
                let follower_state = query.get(app.world(), follower);

                assert!(follower_state.is_ok(), "Cannot get follower state!");

                let follower_state = follower_state.unwrap();
                let current_target = follower_state.target;

                assert!(current_target.is_some(), "Entity has no target!");
                assert_eq!(
                    existing_target, current_target,
                    "Target did not match expected!"
                );

                assert!(
                    !follower_state.should_re_path(),
                    "Re-path trigger should not occur!"
                );
            }

            #[test]
            fn gain_target_not_following() {
                // GIVEN
                // An entity in the following state without a target
                let TargetTestFixture {
                    mut app, follower, ..
                } = setup_targets(false);

                // WHEN
                // We remove it from the following state (this tests the component removal as well)
                app.world_mut().entity_mut(follower).remove::<Following>();

                // And then try to assign it a follower target
                let expected_target = app.world_mut().spawn_empty().id();
                app.world_mut().trigger(GainedTarget {
                    entity: follower,
                    target: expected_target,
                });

                app.update();

                // THEN
                // An error should occur and no changes should be made
                let mut query = app.world_mut().query::<&FollowerState>();
                let follower_state = query.get(app.world(), follower);

                assert!(
                    follower_state.is_err(),
                    "Follower state should not be present!"
                );

                let mut query = app.world_mut().query::<&GainLoseTargetError>();
                let error = query.get(app.world(), follower);

                assert!(error.is_ok(), "Error should be present!");
            }

            #[test]
            fn gain_target_no_follow_data() {
                // GIVEN
                // An entity without follower data
                let TargetTestFixture { mut app, .. } = setup_targets(false);
                let non_follower = app.world_mut().spawn_empty().id();

                // WHEN
                // We try to assign it a follower target
                let expected_target = app.world_mut().spawn_empty().id();
                app.world_mut().trigger(GainedTarget {
                    entity: non_follower,
                    target: expected_target,
                });

                app.update();

                // THEN
                // An error should occur and no changes should be made
                let mut query = app.world_mut().query::<(
                    Option<&FollowerData>,
                    Option<&Following>,
                    Option<&FollowerState>,
                )>();
                let (data, marker, state) = query.get(app.world(), non_follower).unwrap();

                assert!(data.is_none(), "Follower data should not be present!");
                assert!(marker.is_none(), "Following marker should not be present!");
                assert!(state.is_none(), "Follower state should not be present!");

                let mut query = app.world_mut().query::<&GainLoseTargetError>();
                let error = query.get(app.world(), non_follower);

                assert!(error.is_ok(), "Error should be present!");
            }
        }

        mod lose_target {
            use super::*;
            use crate::characters::npc::ai::pathfinding::pathfinder::Waypoints;
            use crate::characters::npc::ai::pathfinding::pathfinder_test_components;

            fn setup_path() -> Waypoints {
                Waypoints::new(vec![Vec3::ZERO.into()])
            }

            #[test]
            fn lose_target() {
                // GIVEN
                // An entity in the following state with a target
                let TargetTestFixture {
                    mut app, follower, ..
                } = setup_targets(true);
                app.world_mut()
                    .entity_mut(follower)
                    .insert((setup_path(), pathfinder_test_components()));

                // WHEN
                // We remove that target
                app.world_mut().trigger(LostTarget { entity: follower });
                app.update();

                // THEN
                // The target should be removed and any existing path should be cleared
                let mut query = app.world_mut().query::<&FollowerState>();
                let follower_state = query.get(app.world(), follower);

                assert!(follower_state.is_ok(), "Cannot get follower state!");

                let follower_state = follower_state.unwrap();
                let current_target = follower_state.target;

                assert!(current_target.is_none(), "Target should be removed!");

                let mut query = app.world_mut().query::<&Waypoints>();
                let waypoints = query.get(app.world(), follower);

                assert!(waypoints.is_err(), "Waypoints should be removed!");
            }

            #[test]
            fn lose_target_no_target() {
                // GIVEN
                // An entity in the following state no target
                let TargetTestFixture {
                    mut app, follower, ..
                } = setup_targets(false);

                // WHEN
                // We try to remove its target
                app.world_mut().trigger(LostTarget { entity: follower });
                app.update();

                // THEN
                // Nothing should happen
                let mut query = app.world_mut().query::<&FollowerState>();
                let follower_state = query.get(app.world(), follower);

                assert!(follower_state.is_ok(), "Cannot get follower state!");

                let follower_state = follower_state.unwrap();
                let current_target = follower_state.target;

                assert!(current_target.is_none(), "Target should not be present!");
            }

            #[test]
            fn lose_target_not_following() {
                // GIVEN
                // An entity not in the following state
                let TargetTestFixture {
                    mut app, follower, ..
                } = setup_targets(false);
                app.world_mut().entity_mut(follower).remove::<Following>();

                // WHEN
                // We try to remove its target
                app.world_mut().trigger(LostTarget { entity: follower });
                app.update();

                // THEN
                // An error should occur and no changes should be made
                let mut query = app.world_mut().query::<&FollowerState>();
                let follower_state = query.get(app.world(), follower);

                assert!(
                    follower_state.is_err(),
                    "Follower state should not be present!"
                );

                let mut query = app.world_mut().query::<&GainLoseTargetError>();
                let error = query.get(app.world(), follower);

                assert!(error.is_ok(), "Error should be present!");
            }

            #[test]
            fn lose_target_no_follow_data() {
                // GIVEN
                // An entity without follower data
                let TargetTestFixture { mut app, .. } = setup_targets(false);
                let non_follower = app.world_mut().spawn_empty().id();

                // WHEN
                // We try to remove its target
                app.world_mut().trigger(LostTarget {
                    entity: non_follower,
                });
                app.update();

                // THEN
                // An error should occur and no changes should be made
                let mut query = app.world_mut().query::<(
                    Option<&FollowerData>,
                    Option<&Following>,
                    Option<&FollowerState>,
                )>();
                let (data, marker, state) = query.get(app.world(), non_follower).unwrap();

                assert!(data.is_none(), "Follower data should not be present!");
                assert!(marker.is_none(), "Following marker should not be present!");
                assert!(state.is_none(), "Follower state should not be present!");

                let mut query = app.world_mut().query::<&GainLoseTargetError>();
                let error = query.get(app.world(), non_follower);

                assert!(error.is_ok(), "Error should be present!");
            }
        }
    }
}
