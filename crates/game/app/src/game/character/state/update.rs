use crate::game::character::state::capabilities::ActionStateCapabilities;
use crate::game::character::state::states::Idle;
use crate::game::character::state::tracking::{ActionStateEvent, ActionStateTracker, ReflectTimedActionState};
use crate::game::character::Character;
use crate::prelude::*;
use tracing::error;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (update_timed_state,).in_set(AppSystems::Update));
}

pub fn update_timed_state(
    time: Res<Time>,
    mut commands: Commands,
    registry: Res<AppTypeRegistry>,
    query: Query<(Entity, &ActionStateTracker, &ActionStateCapabilities), With<Character>>,
) {
    let delta = time.delta_secs();
    let type_registry = registry.read();

    for (entity, tracker, state_capabilities) in &query {
        // Find the type registration for the current state
        let Some(registration) = type_registry.get(tracker.type_id) else {
            continue;
        };
        let Some(_) = registration.data::<ReflectTimedActionState>() else {
            continue;
        };
        let Some(_) = registration.data::<ReflectComponent>() else {
            continue;
        };

        let type_id = tracker.type_id;
        let state_capabilities = state_capabilities.clone();

        // Perform the update via command queue to get EntityWorldMut
        commands.queue(move |world: &mut World| {
            let type_registry = world.resource::<AppTypeRegistry>().clone();
            let type_registry = type_registry.read();

            // Re-fetch helpers inside closure
            let reg = type_registry.get(type_id).unwrap();
            let reflect_timed_state = reg.data::<ReflectTimedActionState>().unwrap();
            let reflect_component = reg.data::<ReflectComponent>().unwrap();

            if let Ok(mut entity_mut) = world.get_entity_mut(entity)
                && let Some(reflect_data) = reflect_component.reflect_mut(&mut entity_mut)
                && let Some(timed_state) = reflect_timed_state.get_mut(reflect_data.into_inner())
            {
                let new_time = timed_state.time_left() - delta;
                timed_state.set_time(new_time);

                if new_time > 0.0 {
                    return;
                }

                let prev_data = timed_state.box_clone();

                match ActionStateEvent::try_new(
                    entity,
                    &state_capabilities,
                    Box::new(Idle),
                    prev_data,
                ) {
                    Ok(event) => {
                        world.commands().trigger(event);
                    }
                    Err(_) => error!("Failed to transition to Idle state for entity {}", entity),
                }
            }
        });
    }
}