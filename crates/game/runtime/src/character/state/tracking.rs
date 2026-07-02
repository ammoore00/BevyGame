use crate::character::Character;
use assets::action_states::{ActionState, ActionStateCapabilities, Idle, ReflectActionState, ReflectMovementActionState, StateTransitionError};
use bevy::prelude::*;
use std::any::TypeId;
use std::fmt::Debug;
use std::sync::Arc;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_state_change);
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ActionStateTracker {
    pub(crate) type_id: TypeId,
}

impl Default for ActionStateTracker {
    fn default() -> Self {
        Self {
            type_id: TypeId::of::<Idle>(),
        }
    }
}

// TODO: Reduce code duplication in these functions

pub fn get_state(
    entity: Entity,
    tracker: &ActionStateTracker,
    world: &World,
) -> Option<Box<dyn ActionState>> {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let reg = type_registry.get(tracker.type_id).unwrap();
    let reflect_component = reg.data::<ReflectComponent>().unwrap();
    let reflect_state = reg.data::<ReflectActionState>().unwrap();

    if let Ok(entity) = world.get_entity(entity)
        && let Some(reflect_data) = reflect_component.reflect(&entity)
        && let Some(state) = reflect_state.get(reflect_data)
    {
        Some(state.box_clone())
    } else {
        warn!("Failed to get reflect component for entity {}", entity);
        None
    }
}

pub fn is_in_movement_state(
    entity: Entity,
    tracker: &ActionStateTracker,
    world: &World,
) -> bool {
    let registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = registry.read();
    let reg = type_registry.get(tracker.type_id).unwrap();
    let reflect_component = reg.data::<ReflectComponent>().unwrap();

    if let Ok(entity) = world.get_entity(entity)
        && let Some(reflect_data) = reflect_component.reflect(entity)
        && let Some(reflect_movement_state) = reg.data::<ReflectMovementActionState>()
    {
        reflect_movement_state.get(reflect_data).is_some()
    } else {
        false
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SetStateError {
    #[error(transparent)]
    Transition(#[from] StateTransitionError),
    #[error("Failed to retrieve state tracker component")]
    StateTracker,
    #[error("Failed to retrieve previous state component")]
    PrevState,
    #[error("Failed to update state")]
    StateUpdate,
}


/// Trigger to attempt to set the state of the provided entity to the new state.
///
/// Entity must have the following components:
/// - `ActionStateTracker`
/// - `ActionStateCapabilities`
/// - Some variant of `ActionState`
///
/// Fails if any required component is missing or if the state transition is invalid.
///
/// Use `with_callback` to provide a callback to be invoked when the state transition is invoked.
/// The callback parameters include the result of the state transition.
#[derive(EntityEvent)]
pub struct TrySetStateEvent {
    entity: Entity,
    state: Box<dyn ActionState>,
    // Arc is used to allow for passing the callback into command queue closure
    callback: Option<Arc<Box<dyn StateEventCallback>>>,
}
impl TrySetStateEvent {
    pub fn new(entity: Entity, state: Box<dyn ActionState>) -> Self {
        Self { entity, state, callback: None }
    }

    pub fn with_callback<Callback: StateEventCallback + 'static>(self, callback: Callback) -> Self {
        Self { callback: Some(Arc::new(Box::new(callback))), ..self }
    }
}

pub trait StateEventCallback: Fn(Entity, Commands, Result<(), SetStateError>) + Send + Sync + 'static {}
impl<T> StateEventCallback for T
where T: Fn(Entity, Commands, Result<(), SetStateError>) + Send + Sync + 'static {}

pub fn on_state_change(
    event: On<TrySetStateEvent>,
    mut commands: Commands
) {
    let entity = event.entity;
    let new_state = event.state.box_clone();
    let callback = event.callback.clone();

    // Use the queue to get full World access after the observer logic
    commands.queue(move |world: &mut World| {
        // Query the world to get the components necessary to set the state
        let mut entity_query = world.query_filtered::<
            (
                &ActionStateTracker,
                &ActionStateCapabilities,
            ),
            With<Character>,
        >();
        let (tracker, capabilities) = match entity_query.get(world, entity) {
            Ok((tracker, capabilities)) => (tracker, capabilities),
            Err(err) => {
                if let Some(callback) = callback {
                    callback(entity, world.commands(), Err(SetStateError::StateTracker));
                }
                error!("Failed to get action state tracker for entity: {}", err);
                return;
            }
        };

        // Get the entity's previous state
        let Some(prev_state) = get_state(entity, tracker, world) else {
            if let Some(callback) = callback {
                callback(entity, world.commands(), Err(SetStateError::PrevState));
            }
            error!("Failed to get previous action state for entity {}", entity);
            return;
        };

        // Validate if the transition is allowed
        if let Err(err) =  capabilities.can_transition(prev_state.as_ref(), new_state.as_ref()) {
            if let Some(callback) = callback {
                callback(entity, world.commands(), Err(err.into()));
            }
            return;
        };

        // Use reflection to perform the state transition
        let new_type_id = (*new_state).type_id();
        let prev_type_id = prev_state.type_id();

        let registry = world.resource::<AppTypeRegistry>().clone();
        let type_registry = registry.read();

        if let Some(prev_type) = type_registry.get(prev_type_id)
            && let Some(prev_reflect_component) = prev_type.data::<ReflectComponent>()
            && let Some(next_type) = type_registry.get(new_type_id)
            && let Some(next_reflect_component) = next_type.data::<ReflectComponent>()
            && let Ok(mut entity_mut) = world.get_entity_mut(entity)
        {
            // Remove the old state
            prev_reflect_component.remove(&mut entity_mut);

            // Insert the new state
            next_reflect_component.insert(&mut entity_mut, new_state.as_reflect(), &type_registry);

            // Update the tracker component
            entity_mut.insert(ActionStateTracker {
                type_id: new_type_id,
            });
        } else {
            if let Some(callback) = callback {
                callback(entity, world.commands(), Err(SetStateError::StateUpdate));
            }
            error!("Failed to update state for entity {}: ", entity);
        }
    });
}