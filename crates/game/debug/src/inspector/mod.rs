mod impls;
mod readout;
mod window;

use crate::inspector::readout::InspectorReadout;
use bevy::ecs::query::ArchetypeFilter;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((impls::plugin, window::plugin));

    app.init_state::<InspectorWindowOpen>();
}

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct InspectorWindowOpen(bool);

trait InspectorRegistrar {
    fn add_data_inspector<F: ArchetypeFilter>(
        &mut self,
        inspector_fn: fn(EntityRef, &DeferredWorld) -> InspectorReadout,
    );
}
impl InspectorRegistrar for App {
    fn add_data_inspector<F: ArchetypeFilter>(
        &mut self,
        inspector_fn: fn(EntityRef, &DeferredWorld) -> InspectorReadout,
    ) {
        // Get or create the inspector registry
        let mut registry = match self.world_mut().get_resource_mut::<InspectorRegistry>() {
            Some(registry) => registry,
            None => {
                self.insert_resource(InspectorRegistry::default());
                self.world_mut().resource_mut::<InspectorRegistry>()
            }
        };

        registry
            .registry
            .push(DataInspector::new::<F>(inspector_fn));
    }
}

#[derive(Resource, Default, Clone)]
struct InspectorRegistry {
    registry: Vec<DataInspector>,
}
impl InspectorRegistry {
    fn get_matching(
        &self,
        entity: Entity,
        world: &DeferredWorld,
    ) -> Result<DataInspector, InspectorMatchError> {
        let matched = self
            .registry
            .iter()
            .filter(|inspector| (inspector.condition_fn)(entity, world))
            .cloned()
            .collect::<Vec<_>>();

        match matched.len() {
            0 => Err(InspectorMatchError::NoMatch),
            1 => Ok(matched[0]),
            _ => Err(InspectorMatchError::MultipleMatches),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
enum InspectorMatchError {
    #[error("No match found")]
    NoMatch,
    #[error("Multiple matches found")]
    MultipleMatches,
}

#[derive(Clone, Copy)]
struct DataInspector {
    /// Given the current world, does the provided entity trigger the inspector?
    condition_fn: fn(Entity, &DeferredWorld) -> bool,
    /// Retrieve all data from the entity for this inspector and return it as a list of strings to display in debug
    inspector_fn: fn(EntityRef, &DeferredWorld) -> InspectorReadout,
}
impl DataInspector {
    fn new<F: ArchetypeFilter>(
        inspector_fn: fn(EntityRef, &DeferredWorld) -> InspectorReadout,
    ) -> Self {
        Self {
            condition_fn: inspector_matches::<F>,
            inspector_fn,
        }
    }
}

fn inspector_matches<F: ArchetypeFilter>(entity: Entity, world: &DeferredWorld) -> bool {
    let Ok(entity_ref) = world.get_entity(entity) else {
        return false;
    };

    let Some(state) = F::get_state(world.components()) else {
        return false;
    };

    F::matches_component_set(&state, &|component_id| entity_ref.contains_id(component_id))
}
