use bevy::prelude::*;
use common::WorldCoords;

pub(super) fn plugin(app: &mut App) {

}

/// Stores information about the pathfinding target for an NPC
#[derive(Component, Debug, Clone)]
pub enum TargetGoal {
    /// A static target position
    Position {
        /// The target position
        coords: WorldCoords,
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
        coarse_threshold: Option<f32>,
    }
}
impl TargetGoal {
    fn position(coords: WorldCoords, threshold_dist: f32) -> Self {
        Self::Position { coords, threshold_dist }
    }

    fn entity(entity: Entity, threshold_dist: f32) -> Self {
        Self::Entity { entity, threshold_dist, coarse_threshold: None }
    }
}