use crate::characters::npc::ai::pathfinding::pathfinder::{PathfindRequest, PathfinderState, DEFAULT_TARGET_REACHED_THRESHOLD};
use crate::characters::npc::ai::pathfinding::strategy::{
    PathfindStrategy, PathfindStrategyRegistry, ReflectPathfindStrategy,
};
use crate::characters::npc::ai::pathfinding::target::TargetGoal;
use crate::characters::npc::ai::pathfinding::{PathfinderData, PathfinderSystems};
use crate::prelude::Waypoints;
use bevy::prelude::*;
use common::WorldCoords;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        position_dispatch.in_set(PathfinderSystems::Dispatch),
    );

    app.register_pathfind_strategy::<MovingToPos>();

    app.add_observer(on_move_to_pos);
}

#[derive(EntityEvent, Debug, Clone, Copy, derive_new::new)]
pub struct MoveToPosEvent {
    pub entity: Entity,
    pub pos: WorldCoords,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct MoveTargetPos(WorldCoords);

fn on_move_to_pos(event: On<MoveToPosEvent>, mut commands: Commands) {
    commands.entity(event.entity).insert((
        MovingToPos,
        MoveTargetPos(event.pos),
    ));
}

#[derive(Component, Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
#[reflect(Component, PathfindStrategy)]
struct MovingToPos;
impl PathfindStrategy for MovingToPos {}

fn position_dispatch(
    pathfinder_query: Query<(PathfinderData, &MoveTargetPos), (With<MovingToPos>, Without<Waypoints>)>,
    mut commands: Commands,
) {
    for (pathfinder_data, move_target_pos) in pathfinder_query {
        if pathfinder_data.pathfinder.state() != PathfinderState::Dispatch {
            continue;
        }

        let target_goal = pathfinder_data.target_goal
            .as_ref()
            .and_then(|target_goal| {
                if let TargetGoal::Position(current_target) = **target_goal
                    && current_target.pos() == move_target_pos.0
                {
                    Some(**target_goal)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                let target_goal = TargetGoal::position(move_target_pos.0, DEFAULT_TARGET_REACHED_THRESHOLD);
                commands.entity(pathfinder_data.entity).insert(target_goal);
                target_goal
            });

        let TargetGoal::Position(target_pos) = target_goal else {
            unreachable!("If the target goal was not a position, it would be overwritten above");
        };

        let start_loc = WorldCoords::from(pathfinder_data.pos.0 - Vec3::Y);
        let target_loc = WorldCoords::from(target_pos.pos() - Vec3::Y);

        let request = PathfindRequest::new(start_loc, target_loc, pathfinder_data.clearance());
        commands.entity(pathfinder_data.entity).insert(request);
    }
}
