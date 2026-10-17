use crate::inspector::{InspectorReadout, InspectorRegistrar};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use runtime::prelude::Npc;

pub(super) fn plugin(app: &mut App) {
    app.add_data_inspector::<With<Npc>>(inspect_npc);
}

fn inspect_npc(entity: Entity, world: &DeferredWorld) -> InspectorReadout {
   vec!["Npc inspection".to_string()].into()
}
