use bevy::prelude::*;
use common::WorldCoords;
use physics::Collider;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, compute_coarse_threshold);
}

/// Stores information about the pathfinding target for an NPC
#[derive(Component, Debug)]
pub enum TargetGoal {
    /// A static target position
    Position {
        /// The target position
        _coords: WorldCoords,
        /// Threshold distance for detecting when we've reached the target
        threshold_dist: f32,
    },
    /// A target entity with a collider to account for
    Entity {
        /// The target entity
        entity: Entity,
        /// Threshold distance between colliders for when we're close enough to the target
        threshold_dist: f32,
        /// Coarse distance above which we don't use collider checks for performance.
        /// This is equal to the maximum bounds radius of both colliders plus the threshold distance.
        ///
        /// None value means the value has not yet been computed.
        coarse_threshold: CoarseThreshold,
    },
}
impl TargetGoal {
    pub fn position(coords: WorldCoords, threshold_dist: f32) -> Self {
        Self::Position {
            _coords: coords,
            threshold_dist,
        }
    }

    pub fn entity(entity: Entity, threshold_dist: f32) -> Self {
        Self::Entity {
            entity,
            threshold_dist,
            coarse_threshold: None,
        }
    }
    
    // TODO: Replace this method with real collision distance detection
    pub fn threshold_dist(&self) -> f32 {
        match self {
            Self::Position { threshold_dist, .. } => *threshold_dist,
            Self::Entity { coarse_threshold, .. } => coarse_threshold.clone().unwrap().unwrap(),
        }
    }
}

pub type CoarseThreshold = Option<Result<f32, ThresholdError>>;

fn compute_coarse_threshold(
    pathfinder_query: Query<(&mut TargetGoal, Option<&Collider>)>,
    target_entity_query: Query<&Collider>,
) {
    for (mut target_goal, this_collider) in pathfinder_query {
        let TargetGoal::Entity {
            entity: target,
            threshold_dist,
            ref mut coarse_threshold,
        } = *target_goal
        else {
            continue;
        };

        let this_bound = this_collider
            .map(|collider| collider.max_bound_radius())
            .unwrap_or_default();
        let other_bound = target_entity_query
            .get(target)
            .map(|collider| collider.max_bound_radius())
            .unwrap_or_default();

        *coarse_threshold = Some(Ok(this_bound + other_bound + threshold_dist));
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ThresholdError {
}
