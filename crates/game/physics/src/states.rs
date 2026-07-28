use bevy::prelude::*;
use common::{AppSystems, GameplaySystems, PausableSystems};

pub(crate) fn plugin(app: &mut App) {
    app.configure_sets(
        FixedUpdate,
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
            .run_if(in_state(PhysicsLoaded(true)))
            .in_set(GameplaySystems)
            .in_set(PausableSystems)
            .in_set(AppSystems::Update),
    );

    app.init_state::<PhysicsLoaded>();

    app.add_observer(on_level_loaded);
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum PhysicsPipeline {
    DetectCollisions,
    ApplyIntent,
    ReactToForces,
    RespondToCollisions,
    UpdatePositions,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DetectorCollisionResponse;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicsLevelLoadedEvent(pub bool);

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PhysicsLoaded(bool);

fn on_level_loaded(
    event: On<PhysicsLevelLoadedEvent>,
    mut state: ResMut<NextState<PhysicsLoaded>>
) {
    state.set(PhysicsLoaded(event.0));
}