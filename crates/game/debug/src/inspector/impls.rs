use crate::inspector::{InspectorReadout, InspectorRegistrar};
use assets::action_states::ReflectActionState;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use runtime::prelude::{AiState, Health, Npc, Player, ReflectPathfindStrategy};

pub(super) fn plugin(app: &mut App) {
    app.add_data_inspector::<With<Npc>>(inspect_npc);
    app.add_data_inspector::<With<Player>>(inspect_player);
}

fn inspect_npc(entity: EntityRef, world: &DeferredWorld) -> InspectorReadout {
    let mut npc_out = Vec::new();

    append_reflected_component::<ReflectPathfindStrategy>(entity, world, &mut npc_out, |name| {
        format!("Pathfind Strategy: {}", name)
    });
    append_inspection::<AiState>(entity, &mut npc_out, |ai_state| {
        format!("AI State: {}", ai_state.current())
    });

    let mut total_out = inspect_character(entity, world);
    total_out.append(&mut npc_out);
    total_out.into()
}

fn inspect_player(entity: EntityRef, world: &DeferredWorld) -> InspectorReadout {
    inspect_character(entity, world).into()
}

fn inspect_character(entity: EntityRef, world: &DeferredWorld) -> Vec<String> {
    let mut out = Vec::new();

    append_inspection::<Name>(entity, &mut out, |name| {
        format!("Name: {}", name.to_string())
    });
    append_inspection::<Health>(entity, &mut out, |health| {
        format!("Health: {} (max {})", health.current, health.max)
    });
    append_reflected_component::<ReflectActionState>(entity, world, &mut out, |name| {
        format!("Action State: {}", name)
    });

    out
}

fn append_inspection<T: Component>(
    entity: EntityRef,
    out: &mut Vec<String>,
    op: impl FnOnce(&T) -> String,
) {
    entity.get::<T>().inspect(|value| out.push(op(value)));
}

fn append_reflected_component<TraitRegistration: Clone + Send + Sync + 'static>(
    entity: EntityRef,
    world: &DeferredWorld,
    out: &mut Vec<String>,
    op: impl FnOnce(String) -> String,
) {
    reflected_trait_component_name::<TraitRegistration>(entity, world)
        .inspect(|name| out.push(op(name.clone())));
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
