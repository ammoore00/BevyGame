use bevy::prelude::*;
use common::{AppSystems, GameplaySystems, PausableSystems};

pub(crate) fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (
            (
                PhysicsPipeline::DetectCollisions,
                PhysicsPipeline::ApplyIntent,
                PhysicsPipeline::ReactToForces,
                PhysicsPipeline::RespondToCollisions,
                PhysicsPipeline::UpdatePositions,
            )
                .chain(),
            (PhysicsPipeline::DetectCollisions, DetectorCollisionResponse).chain(),
        )
            .in_set(GameplaySystems)
            .in_set(PausableSystems)
            .in_set(AppSystems::Update),
    );
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum PhysicsPipeline {
    DetectCollisions,
    ApplyIntent,
    ReactToForces,
    RespondToCollisions,
    UpdatePositions,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct DetectorCollisionResponse;
