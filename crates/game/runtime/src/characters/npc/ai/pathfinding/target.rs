use bevy::prelude::*;
use common::WorldCoords;
use getset::CopyGetters;
use physics::Collider;

pub(super) fn plugin(_app: &mut App) {}

/// Stores information about the pathfinding target for an NPC
#[derive(Component, Debug, Clone, Copy)]
pub enum TargetGoal {
    /// A static target position
    Position(TargetPosition),
    /// A target entity with a collider to account for
    Entity(TargetEntity),
}
impl TargetGoal {
    pub fn position(coords: WorldCoords, threshold_dist: f32) -> Self {
        Self::Position(TargetPosition::new(coords, threshold_dist))
    }

    pub fn entity(entity: Entity, threshold_dist: f32) -> Self {
        Self::Entity(TargetEntity::new(entity, threshold_dist))
    }
}

#[derive(Debug, Clone, Copy, derive_new::new, CopyGetters)]
pub struct TargetPosition {
    /// The target position
    #[getset(get_copy = "pub")]
    pos: WorldCoords,
    /// Threshold distance for detecting when we've reached the target
    #[getset(get_copy = "pub")]
    threshold_dist: f32,
}

#[derive(Debug, Clone, Copy, derive_new::new, CopyGetters)]
pub struct TargetEntity {
    /// The target entity
    #[getset(get_copy = "pub")]
    entity: Entity,
    /// Threshold distance between colliders for when we're close enough to the target
    threshold_dist: f32,
}
impl TargetEntity {
    pub fn is_within_threshold(&self, first: &Collider, second: &Collider) -> bool {
        let coarse_threshold =
            first.max_bound_radius() + second.max_bound_radius() + self.threshold_dist;

        first
            .distance(second, coarse_threshold)
            .map(|collision_data| collision_data.dist() < self.threshold_dist)
            .unwrap_or_default()
    }
}
