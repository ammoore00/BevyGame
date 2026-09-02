use crate::commands::parser::{CommandRegistrar, DebugCommand};
use bevy::prelude::*;
use winnow::ModalResult;

pub(super) fn plugin(app: &mut App) {
    app.add_debug_command::<CharacterCommand>();
}

struct CharacterCommand;
impl DebugCommand for CharacterCommand {
    const NAME: &'static str = "character";

    fn parse(input: &mut &str) -> ModalResult<Box<Self>> {
        Ok(Box::new(CharacterCommand))
    }

    fn invoke(&self, world: &mut World) -> String {
        "Character Command Invoked".to_string()
    }
}
