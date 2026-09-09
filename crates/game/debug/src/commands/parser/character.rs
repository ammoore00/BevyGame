use crate::commands::CommandPickable;
use crate::commands::parser::basic_parsers::{parse_digits, parse_uuid};
use crate::commands::parser::{CommandRegistrar, DebugCommand};
use crate::commands::window::CommandsWindowOpen;
use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use common::marker;
use runtime::characters::{Character, DeathEvent};
use runtime::debug::Health;
use std::convert::Infallible;
use std::fmt::Display;
use std::ops::{Add, Sub};
use strum_macros::Display;
use winnow::ascii::{digit1, space1};
use winnow::combinator::{alt, fail, preceded};
use winnow::error::{StrContext, StrContextValue};
use winnow::{ModalResult, Parser};

pub(super) fn plugin(app: &mut App) {
    app.add_debug_command::<CharacterCommand>();

    app.add_systems(OnEnter(CommandsWindowOpen(true)), add_pickable);
    app.add_systems(OnEnter(CommandsWindowOpen(false)), remove_pickable);
}

/// # Character
/// Syntax: `character <entity> <operation>`
///
/// ### Operations
/// - `kill`
/// - `attribute`
///
/// ##### Kill
/// Kills the target character
///
/// No additional arguments
///
/// ##### Attribute
/// Modify attributes of the target character
///
/// `attribute <attr> <set|add|sub> <amount>`
#[derive(Debug)]
struct CharacterCommand {
    target: CharacterCommandTarget,
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
            target: CharacterCommandTarget::Uuid(entity_id),
            operation,
        }))
    }

    fn invoke(&self, world: &mut World) -> Result<String, Self::Err> {
        let targets = self.target.get_entities(world);
        let target_count = targets.len();

        let out = match self.operation {
            CharacterOperation::Kill => {
                targets
                    .iter()
                    .for_each(|entity| world.trigger(DeathEvent::new(*entity)));
                format!("Killed {target_count} character(s)")
            }
            CharacterOperation::Attribute(attr) => match attr {
                Attribute::Health { operation, amount } => {
                    targets.iter().for_each(|entity| {
                        let mut query = world.query::<&mut Health>();
                        if let Ok(mut health) = query.get_mut(world, *entity) {
                            health.current = operation.apply(health.current, amount);
                        }
                    });
                    format!(
                        "{} for {target_count} character(s)",
                        operation.display_output(amount, "health")
                    )
                }
            },
            CharacterOperation::Ai(_) => "Not yet implemented!".to_string(),
        };

        Ok(out)
    }
}

#[derive(Debug)]
enum CharacterCommandTarget {
    Uuid(Uuid),
}
impl CharacterCommandTarget {
    fn get_entities(&self, world: &mut World) -> Vec<Entity> {
        match self {
            CharacterCommandTarget::Uuid(uuid) => {
                let mut query = world.query::<(Entity, &CommandPickable)>();
                query
                    .iter(world)
                    .filter(|(_, u)| u.0 == *uuid)
                    .map(|(e, _)| e)
                    .collect()
            }
        }
    }
}

fn parse_operation(input: &mut &str) -> ModalResult<CharacterOperation> {
    info!("Parsing operation from: '{}'", input);
    alt((
        ("kill", ()).map(|_| CharacterOperation::Kill),
        ("attribute", preceded(space1, parse_attribute))
            .map(|(_, operation)| operation)
            .context(StrContext::Label("attribute <attribute>")),
        ("ai", preceded(space1, parse_ai_operation))
            .map(|(_, operation)| operation)
            .context(StrContext::Label("ai <operation>")),
        fail.context(StrContext::Label("operation"))
            .context(StrContext::Expected(StrContextValue::StringLiteral(
                "attribute",
            ))),
    ))
    .parse_next(input)
}

#[derive(Debug, Display)]
enum CharacterOperation {
    Kill,
    Attribute(Attribute),
    Ai(AiOperation),
}

fn parse_attribute(input: &mut &str) -> ModalResult<CharacterOperation> {
    Ok(CharacterOperation::Attribute(
        alt(((
            "health",
            preceded(
                space1,
                (
                    parse_attribute_operation,
                    preceded(space1, parse_digits::<u32>),
                ),
            ),
        )
            .map(|(_, (operation, amount))| Attribute::Health { operation, amount }),))
        .parse_next(input)?,
    ))
}

#[derive(Debug, Clone, Copy)]
enum Attribute {
    Health {
        operation: AttributeOperation,
        amount: u32,
    },
}

fn parse_attribute_operation(input: &mut &str) -> ModalResult<AttributeOperation> {
    alt((
        ("set", ()).map(|_| AttributeOperation::Set),
        ("add", ()).map(|_| AttributeOperation::Add),
        ("sub", ()).map(|_| AttributeOperation::Sub),
    ))
    .parse_next(input)
}

#[derive(Debug, Clone, Copy)]
enum AttributeOperation {
    Set,
    Add,
    Sub,
}
impl AttributeOperation {
    fn apply<T>(&self, initial: T, amount: T) -> T
    where
        T: Add<Output = T> + Sub<Output = T>,
    {
        match self {
            AttributeOperation::Set => amount,
            AttributeOperation::Add => initial + amount,
            AttributeOperation::Sub => initial - amount,
        }
    }

    fn display_output<T: Display>(&self, amount: T, attr: impl AsRef<str>) -> String {
        let attr = attr.as_ref();
        match self {
            AttributeOperation::Set => format!("Set {attr} to {amount}"),
            AttributeOperation::Add => format!("Added {amount} {attr}"),
            AttributeOperation::Sub => format!("Subtracted {amount} {attr}"),
        }
    }
}

fn parse_ai_operation(input: &mut &str) -> ModalResult<CharacterOperation> {
    todo!()
}

#[derive(Debug, Clone, Copy)]
enum AiOperation {
    Enable,
    Disable,
    SetMode(AiMode),
}

#[derive(Debug, Clone, Copy)]
enum AiMode {
    Follow,
    Wander,
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
