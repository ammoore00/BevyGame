use assets::action_states::{ActionState, ActionStateCapabilities, Idle, ReflectActionState, ReflectMovementActionState, StateTransitionError};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use std::any::TypeId;
use std::fmt::Debug;
use tracing::warn;

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

pub fn get_state(
    entity: Entity,
    tracker: &ActionStateTracker,
    world: &mut World,
) -> Option<Box<dyn ActionState>> {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let reg = type_registry.get(tracker.type_id).unwrap();
    let reflect_component = reg.data::<ReflectComponent>().unwrap();
    let reflect_state = reg.data::<ReflectActionState>().unwrap();

    if let Ok(mut entity_mut) = world.get_entity_mut(entity)
        && let Some(reflect_data) = reflect_component.reflect_mut(&mut entity_mut)
        && let Some(state) = reflect_state.get_mut(reflect_data.into_inner())
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
    world: &mut World,
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

/// Attempts to set the state of the provided entity to the new state, triggering a state event if successful.
/// 
/// Entity must have the following components:
/// - `ActionStateTracker`
/// - `ActionStateCapabilities`
/// - Some variant of `ActionState`
/// 
/// Returns an error if any required component is missing or if the state transition is invalid.
pub fn try_set_state(
    entity: Entity,
    new_state: Box<dyn ActionState>,
    world: &mut World
) -> Result<(), SetStateError> {
    let state_tracker = *world.get::<ActionStateTracker>(entity)
        .ok_or(SetStateError::StateTracker)?;
    
    let prev_state = get_state(entity, &state_tracker, world)
        .ok_or(SetStateError::PrevState)?;

    let state_capabilities = world.get::<ActionStateCapabilities>(entity)
        .cloned()
        .ok_or(SetStateError::Capabilities)?;

    let state_event = ActionStateEvent::try_new(entity, &state_capabilities, new_state, prev_state)
        .map_err(SetStateError::from)?;
    
    world.trigger(state_event);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SetStateError {
    #[error(transparent)]
    Transition(#[from] StateTransitionError),
    #[error("Failed to retrieve state tracker component")]
    StateTracker,
    #[error("Failed to retrieve previous state component")]
    PrevState,
    #[error("Failed to retrieve state capabilities component")]
    Capabilities,
}

#[derive(EntityEvent, Debug)]
pub struct ActionStateEvent {
    entity: Entity,
    new_state: Box<dyn ActionState>,
    prev_state: Box<dyn ActionState>,
}

impl ActionStateEvent {
    pub fn try_new(
        entity: Entity,
        transitions: &ActionStateCapabilities,
        new_state: Box<dyn ActionState>,
        prev_state: Box<dyn ActionState>,
    ) -> Result<Self, StateTransitionError> {
        transitions.can_transition(prev_state.as_ref(), new_state.as_ref())?;

        Ok(Self {
            entity,
            new_state,
            prev_state,
        })
    }
}

pub fn on_state_change(event: On<ActionStateEvent>, mut world: DeferredWorld) {
    let entity = event.entity;

    // We clone these to move them into the command closure
    let new_state = event.new_state.clone_value();
    let prev_type_id = (*event.prev_state).type_id();
    let new_type_id = (*new_state).type_id();

    // Use the queue to get full World access after the observer logic
    world.commands().queue(move |world: &mut World| {
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
            warn!("Failed to update state for entity {}: ", entity);
        }
    });
}