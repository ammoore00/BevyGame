use crate::commands::CommandPickable;
use crate::commands::parser::basic_parsers::{parse_digits, parse_entity, parse_prefix};
use crate::commands::parser::{CommandRegistrar, DebugCommand};
use crate::commands::window::CommandsWindowOpen;
use bevy::prelude::*;
use common::marker;
use runtime::characters::Character;
use std::convert::Infallible;
use strum_macros::{Display, EnumString};
use winnow::ascii::alpha1;
use winnow::combinator::{alt, fail};
use winnow::error::{StrContext, StrContextValue};
use winnow::{ModalResult, Parser};

pub(super) fn plugin(app: &mut App) {
    app.add_debug_command::<CharacterCommand>();

    app.add_systems(OnEnter(CommandsWindowOpen(true)), add_pickable);
    app.add_systems(OnEnter(CommandsWindowOpen(false)), remove_pickable);
}

#[derive(Debug)]
struct CharacterCommand {
    entity: Entity,
    operation: Operation,
}

impl DebugCommand for CharacterCommand {
    const NAME: &'static str = "character";
    type Err = Infallible;

    fn parse(input: &mut &str) -> ModalResult<Box<Self>> {
        info!("Parsing: {}", input);
        let entity = parse_entity
            .context(StrContext::Label("entity"))
            .context(StrContext::Expected(StrContextValue::Description("Failed to find target entity")))
            .parse_next(input)?;
        let operation = parse_operation(input)?;
        Ok(Box::new(CharacterCommand { entity, operation }))
    }

    fn invoke(&self, _world: &mut World) -> Result<String, Self::Err> {
        Ok(format!("{self:?}"))
    }
}

fn parse_operation(input: &mut &str) -> ModalResult<Operation> {
    alt((
        ("modify", parse_attribute)
            .map(|(_, attr)| Operation::Modify(attr))
            .context(StrContext::Label("modify attribute")),
        fail.context(StrContext::Label("operation"))
            .context(StrContext::Expected(StrContextValue::StringLiteral(
                "modify",
            ))),
    ))
    .parse_next(input)
}

#[derive(Debug, Display)]
enum Operation {
    Modify(Attribute),
}

fn parse_attribute(input: &mut &str) -> ModalResult<Attribute> {
    alt((("modify", alpha1).map(|_| Attribute::Health),)).parse_next(input)
}

#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
enum Attribute {
    Health,
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
