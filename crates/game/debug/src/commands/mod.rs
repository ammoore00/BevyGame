use bevy::asset::uuid::Uuid;
use crate::commands::window::{AddTextEvent, CommandsWindowOpen};
use bevy::prelude::*;
use common::marker;

mod parser;
mod window;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((parser::plugin, window::plugin));

    app.add_observer(on_picked.run_if(in_state(CommandsWindowOpen(true))));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CommandPickable(pub Uuid);
impl CommandPickable {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

fn on_picked(
    event: On<Pointer<Press>>,
    pickable_query: Query<&CommandPickable>,
    mut commands: Commands,
) {
    if let Ok(command_id) = pickable_query.get(event.entity) {
        commands.trigger(AddTextEvent(command_id.0.to_string()));
    }
}
