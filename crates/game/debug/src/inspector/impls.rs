use crate::inspector::InspectorRegistrar;
use crate::inspector::readout::{InspectorReadout, ReadoutSection};
use assets::action_states::ReflectActionState;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use runtime::prelude::{AiState, Health, Npc, Player, ReflectPathfindStrategy, SensedTargets};

pub(super) fn plugin(app: &mut App) {
    app.add_data_inspector::<With<Npc>>(inspect_npc);
    app.add_data_inspector::<With<Player>>(inspect_player);
}

fn inspect_npc(entity: EntityRef, world: &DeferredWorld) -> InspectorReadout {
    InspectorReadout::new("NPC")
        .section({
            ReadoutSection::new("AI")
                .maybe_item(
                    reflected_trait_component_name::<ReflectPathfindStrategy>(entity, world)
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
        })
        .with_extension(inspect_character(entity, world))
}

fn inspect_player(entity: EntityRef, world: &DeferredWorld) -> InspectorReadout {
    inspect_character(entity, world).with_title("Player")
}

fn inspect_character(entity: EntityRef, world: &DeferredWorld) -> InspectorReadout {
    InspectorReadout::new("Character")
        .section({
            let health = entity
                .get::<Health>()
                .expect("Character created without Health component");
            ReadoutSection::new("Health")
                .item(format!("Current: {} / {}", health.current, health.max))
        })
        .section({
            let action_state = reflected_trait_component_name::<ReflectActionState>(entity, world)
                .expect("Character without action state");
            ReadoutSection::new("Action State").item(action_state)
        })
}

fn reflected_trait_component_name<TraitRegistration: Clone + Send + Sync + 'static>(
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
