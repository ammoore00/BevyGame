use crate::commands::CommandPickable;
use crate::commands::parser::basic_parsers::{parse_digits, parse_prefix, parse_uuid};
use crate::commands::parser::{CommandRegistrar, DebugCommand};
use crate::commands::window::CommandsWindowOpen;
use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use common::marker;
use runtime::characters::Character;
use std::convert::Infallible;
use strum_macros::{Display, EnumString};
use winnow::ascii::{alpha1, space1};
use winnow::combinator::{alt, fail, preceded};
use winnow::error::{StrContext, StrContextValue};
use winnow::{ModalResult, Parser};

pub(super) fn plugin(app: &mut App) {
    app.add_debug_command::<CharacterCommand>();

    app.add_systems(OnEnter(CommandsWindowOpen(true)), add_pickable);
    app.add_systems(OnEnter(CommandsWindowOpen(false)), remove_pickable);
}

#[derive(Debug)]
struct CharacterCommand {
    entity_id: Uuid,
    operation: CharacterOperation,
}

impl DebugCommand for CharacterCommand {
    const NAME: &'static str = "character";
    type Err = Infallible;

    fn parse(input: &mut &str) -> ModalResult<Box<Self>> {
        let entity_id = parse_uuid
            .context(StrContext::Label("entity"))
            .context(StrContext::Expected(StrContextValue::Description(
                "Failed to find target entity",
            )))
            .parse_next(input)?;
        let operation = parse_operation(input)?;
        Ok(Box::new(CharacterCommand {
            entity_id,
            operation,
        }))
    }

    fn invoke(&self, _world: &mut World) -> Result<String, Self::Err> {
        Ok(format!("{self:?}"))
    }
}

fn parse_operation(input: &mut &str) -> ModalResult<CharacterOperation> {
    info!("Parsing operation from: '{}'", input);
    alt((
        ("attribute", preceded(space1, parse_attribute))
            .map(|(_, operation)| operation)
            .context(StrContext::Label("attribute <attribute>")),
        fail.context(StrContext::Label("operation"))
            .context(StrContext::Expected(StrContextValue::StringLiteral(
                "attribute",
            ))),
    ))
    .parse_next(input)
}

#[derive(Debug, Display)]
enum CharacterOperation {
    Attribute {
        attribute: Attribute,
        operation: AttributeOperation,
    },
}

fn parse_attribute(input: &mut &str) -> ModalResult<CharacterOperation> {
    let (attribute, operation) =
        alt((("health", ()).map(|_| (Attribute::Health, AttributeOperation::Set)),))
            .parse_next(input)?;
    Ok(CharacterOperation::Attribute {
        attribute,
        operation,
    })
}

#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
enum Attribute {
    Health,
}

#[derive(Debug, Display)]
enum AttributeOperation {
    Set,
    Add,
    Sub,
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

        commands
            .entity(character_entity)
            .insert(CommandPickable::new());
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
