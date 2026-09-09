use bevy::asset::uuid::Uuid;
use crate::commands::window::{AddTextEvent, CommandsWindowOpen};
use bevy::prelude::*;

mod parser;
mod window;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((parser::plugin, window::plugin));

    app.init_resource::<CommandHistory>();

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

#[derive(Resource, Default, Debug)]
pub struct CommandHistory {
    history: Vec<String>,
    /// `None` indicates the user is at the prompt (typing a new command),
    /// while `Some(idx)` points to a specific index in `history`.
    index: Option<usize>,
    stash: Option<String>,
}

impl CommandHistory {
    pub fn next(&mut self) -> String {
        let Some(curr_idx) = self.index else {
            return self.stash.clone().unwrap_or_default();
        };

        if curr_idx < self.history.len() - 1 {
            let next_idx = curr_idx + 1;
            self.index = Some(next_idx);
            self.history[next_idx].clone()
        } else {
            self.index = None;
            self.stash.clone().unwrap_or_default()
        }
    }

    pub fn prev(&mut self) -> String {
        if self.history.is_empty() {
            return self.stash.clone().unwrap_or_default();
        }

        let new_idx = match self.index {
            None => self.history.len() - 1,
            Some(0) => 0,
            Some(idx) => idx - 1,
        };

        self.index = Some(new_idx);
        self.history[new_idx].clone()
    }

    pub fn push_clear(&mut self, command: String) {
        if !command.trim().is_empty() {
            self.history.push(command);
        }
        self.clear_state();
    }

    pub fn clear_state(&mut self) {
        self.index = None;
        self.stash = None;
    }

    pub fn stash(&mut self, command: String) {
        self.stash = Some(command);
    }

    pub fn is_prompt(&self) -> bool {
        self.index.is_none()
    }
}