use crate::commands::CommandPickable;
use crate::commands::parser::{CommandRegistrar, DebugCommand};
use crate::commands::window::CommandsWindowOpen;
use bevy::prelude::*;
use common::marker;
use runtime::characters::Character;
use winnow::ModalResult;

pub(super) fn plugin(app: &mut App) {
    app.add_debug_command::<CharacterCommand>();

    app.add_systems(OnEnter(CommandsWindowOpen(true)), add_pickable);
    app.add_systems(OnEnter(CommandsWindowOpen(false)), remove_pickable);
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

marker!(RemovePickableOnCommandExit);

fn add_pickable(
    character_query: Query<Entity, With<Character>>,
    pickable_query: Query<Entity, With<Pickable>>,
    mut commands: Commands,
) {
    for character_entity in character_query.iter() {
        if pickable_query.get(character_entity).is_err() {
            commands.entity(character_entity).insert((
                RemovePickableOnCommandExit,
                Pickable {
                    should_block_lower: true,
                    is_hoverable: true,
                },
            ));
        }

        commands.entity(character_entity).insert(CommandPickable);
    }
}

fn remove_pickable(
    pickable_query: Query<(Entity, Option<&RemovePickableOnCommandExit>), With<Pickable>>,
    mut commands: Commands,
) {
    for (entity, remove_pickable) in pickable_query.iter() {
        if remove_pickable.is_some() {
            commands
                .entity(entity)
                .remove::<(Pickable, RemovePickableOnCommandExit)>();
        }
        commands.entity(entity).remove::<CommandPickable>();

        info!("Removing Pickable!");
    }
}
