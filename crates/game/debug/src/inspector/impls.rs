use std::any::TypeId;
use crate::inspector::InspectorRegistrar;
use crate::inspector::readout::{InspectorReadout, ReadoutSection};
use assets::action_states::{ReflectActionState, ReflectTimedActionState};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use runtime::prelude::{AiState, GoalList, GoalState, Health, Npc, Player, ReflectPathfindStrategy, SensedTargets};

pub(super) fn plugin(app: &mut App) {
    app.add_data_inspector::<With<Player>>(inspect_player);
    app.add_data_inspector::<With<Npc>>(inspect_npc);
}

fn inspect_player(entity: EntityRef, world: &DeferredWorld) -> InspectorReadout {
    inspect_character(entity, world).with_title("Player")
}

fn inspect_npc(entity: EntityRef, world: &DeferredWorld) -> InspectorReadout {
    InspectorReadout::new("NPC")
        .with_extension(inspect_character(entity, world))
        .section({
            ReadoutSection::new("AI")
                .maybe_item(
                    get_reflected_name::<ReflectPathfindStrategy>(entity, world)
                        .map(|strategy| format!("Pathfinding Strategy: {}", strategy)),
                )
                .maybe_item(
                    entity
                        .get::<AiState>()
                        .map(|ai_state| format!("State: {}", ai_state.current())),
                )
                .maybe_item(
                    entity
                        .get::<SensedTargets>()
                        .map(|targets| targets.targets().len())
                        .map(|count| format!("Sensed Target Count: {}", count)),
                )
                .subsection({
                    let goal_list = entity.get::<GoalList>();
                    let goal_state = entity.get::<GoalState>();
                    
                    let goal_display = goal_list.map(|goals| format!("List: {}", goals));
                    let scores = goal_state
                        .map(|goal_state| {
                            goal_state
                                .scores()
                                .iter()
                                .map(|(goal, score)| {
                                    let score_out = match score {
                                        Some(score) => score.to_string(),
                                        None => "None".to_string(),
                                    };
                                    
                                    format!("{}: {}", goal, score_out)
                                })
                                .collect::<Vec<_>>()
                        })
                        .map(|scores| format!("Scores: {:#?}", scores));

                    ReadoutSection::new("Goals")
                        .maybe_item(goal_display)
                        .maybe_item(scores)
                })
        })
}

fn inspect_character(entity: EntityRef, world: &DeferredWorld) -> InspectorReadout {
    InspectorReadout::new("Character")
        .section({
            let health = entity
                .get::<Health>()
                .expect("Character created without Health component");
            let action_state = get_reflected_name::<ReflectActionState>(entity, world)
                .expect("Character without action state");
            let timed_action_state = get_reflected_component::<ReflectTimedActionState>(entity, world)
                .and_then(|reflect| {
                    let type_registry = world.resource::<AppTypeRegistry>().clone();
                    let type_registry = type_registry.read();
                    
                    for component_id in entity.archetype().components() {
                        // Get the component's info
                        if let Some(info) = world.components().get_info(*component_id)
                            // Get the component's type id
                            && let Some(type_id) = info.type_id()
                            // Get the registration tied to that type id
                            && let Some(registration) = type_registry.get(type_id)
                            // Get the registration for general Components
                            && let Some(reflect_component) = registration.data::<ReflectComponent>()
                            // Get the registration for our specific reflected trait
                            && let Some(reflect_timed_state) = registration.data::<ReflectTimedActionState>()
                            // Get the component itself as a Reflect reference
                            && let Some(reflected_component) = reflect_component.reflect(entity)
                            // Downcast the generic Reflect into the actual trait we are trying to check for
                            && let Some(timed_state) = reflect_timed_state.get(reflected_component)
                        {
                            let time_left = timed_state.time_left().as_secs_f32();
                            let total_time = timed_state.total_time().as_secs_f32();
                            
                            return Some((time_left, total_time))
                        }
                    }
                    
                    None
                });
            
            ReadoutSection::new("Generic")
                .item(format!("Health: {} / {}", health.current, health.max))
                .item(format!("Action State: {}", action_state))
                .maybe_item(timed_action_state.map(|time_left| format!("Time Left in Action: {:.3} / {:.3}", time_left.0, time_left.1)))
        })
}

fn get_reflected_name<TraitRegistration: Clone + Send + Sync + 'static>(
    entity: EntityRef,
    world: &DeferredWorld,
) -> Option<String> {
    let type_registry = world.resource::<AppTypeRegistry>().read();

    for component_id in entity.archetype().components() {
        if let Some(info) = world.components().get_info(*component_id)
            && let Some(type_id) = info.type_id()
            && let Some(registration) = type_registry.get(type_id)
            && registration.data::<TraitRegistration>().is_some()
        {
            let name = info.name().to_string();
            let short_name = name
                .rsplit("::")
                .next()
                .map(|s| s.to_string())
                .unwrap_or(info.name().to_string());

            return Some(short_name);
        }
    }

    None
}

fn get_reflected_component<TraitRegistration: Clone + Send + Sync + 'static>(
    entity: EntityRef,
    world: &DeferredWorld,
) -> Option<TraitRegistration> {
    let type_registry = world.resource::<AppTypeRegistry>().read();
    
    for component_id in entity.archetype().components() {
        if let Some(info) = world.components().get_info(*component_id)
            && let Some(type_id) = info.type_id()
            && let Some(registration) = type_registry.get(type_id)
            && let Some(data) = registration.data::<TraitRegistration>()
        {
            return Some(data.clone());
        }
    }
    
    None
}
