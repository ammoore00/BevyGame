use crate::commands::window::{AddTextEvent, CommandsWindowOpen};
use bevy::prelude::*;
use common::marker;

mod parser;
mod window;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((parser::plugin, window::plugin));

    app.add_observer(on_picked.run_if(in_state(CommandsWindowOpen(true))));
}

marker!(CommandPickable);

fn on_picked(
    event: On<Pointer<Press>>,
    pickable_query: Query<Entity, With<CommandPickable>>,
    mut commands: Commands,
) {
    if let Ok(entity) = pickable_query.get(event.entity) {
        commands.trigger(AddTextEvent(entity.to_string()));
    }
}
