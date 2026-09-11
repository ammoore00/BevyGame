use crate::command_window::CommandPickable;
use crate::command_window::commands::basic_parsers::{parse_digits, parse_uuid};
use crate::command_window::commands::{CommandRegistrar, DebugCommand};
use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use runtime::characters::{Character, DeathEvent};
use runtime::debug::{Following, GainedTarget, Health, Player, Wandering};
use std::convert::Infallible;
use std::fmt::Display;
use std::ops::{Add, Sub};
use strum_macros::Display;
use winnow::ascii::space1;
use winnow::combinator::{alt, fail, opt, preceded};
use winnow::error::{StrContext, StrContextValue};
use winnow::{ModalResult, Parser};

pub(super) fn plugin(app: &mut App) {
    app.add_debug_command::<CharacterCommand>();
}

/// # Character
/// Syntax: `character <operation> <target(s)>`
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
    targets: CharacterCommandTargetList,
    operation: CharacterOperation,
}
impl DebugCommand for CharacterCommand {
    const NAME: &'static str = "character";
    type Err = Infallible;
    type TargetComponent = Character;

    fn parse(input: &mut &str) -> ModalResult<Box<Self>> {
        let operation = parse_operation(input)?;
        let targets = parse_targets(input)?;
        Ok(Box::new(CharacterCommand { targets, operation }))
    }

    fn invoke(&self, world: &mut World) -> Result<String, Self::Err> {
        let targets = self.targets.get_entities(world).into_iter();
        let target_count = targets.len();

        let out = match self.operation {
            CharacterOperation::Kill => {
                targets.for_each(|entity| world.trigger(DeathEvent::new(entity)));
                format!("Killed {target_count} character(s)")
            }
            CharacterOperation::Attribute(attr) => match attr {
                Attribute::Health { operation, amount } => {
                    targets.for_each(|entity| {
                        let mut query = world.query::<&mut Health>();
                        if let Ok(mut health) = query.get_mut(world, entity) {
                            health.current = operation.apply(health.current, amount);
                        }
                    });
                    format!(
                        "{} for {target_count} character(s)",
                        operation.display_output(amount, "health")
                    )
                }
            },
            CharacterOperation::Ai(operation) => match operation {
                AiOperation::Enable => "Not yet implemented!".to_string(),
                AiOperation::Disable => "Not yet implemented!".to_string(),
                AiOperation::Pathfinder(mode) => match mode {
                    PathfinderMode::Follow => {
                        let mut player_query = world.query_filtered::<Entity, With<Player>>();
                        let player = player_query.single(world).expect("Failed to get player!");

                        targets.for_each(|entity| {
                            world
                                .entity_mut(entity)
                                .apply_scene(bsn![@Following])
                                .expect("Failed to apply Following state scene");
                            world.trigger(GainedTarget::new(entity, player));
                        });

                        format!("Set {target_count} character(s) to Following mode")
                    }
                    PathfinderMode::Wander => {
                        targets.for_each(|entity| {
                            world.entity_mut(entity).insert(Wandering);
                        });
                        format!("Set {target_count} character(s) to Wandering mode")
                    }
                },
            },
        };

        Ok(out)
    }
}

fn parse_targets(input: &mut &str) -> ModalResult<CharacterCommandTargetList> {
    let uuid = opt(preceded(space1, parse_uuid)
        .context(StrContext::Label("entity"))
        .context(StrContext::Expected(StrContextValue::Description(
            "Failed to find target entity",
        ))))
    .parse_next(input)?;

    let target = if let Some(entity_id) = uuid {
        CharacterCommandTarget::Uuid(entity_id)
    } else {
        let parse_identifier = alt((("player", ()).map(|_| CharacterCommandIdentifier::Player),))
            .context(StrContext::Label("identifier"))
            .context(StrContext::Expected(StrContextValue::Description(
                "Failed to find target identifier",
            )));

        let identifier = preceded(space1, parse_identifier).parse_next(input)?;
        CharacterCommandTarget::Identifier(identifier)
    };

    Ok(CharacterCommandTargetList::Single(target))
}

#[derive(Debug)]
enum CharacterCommandTargetList {
    Single(CharacterCommandTarget),
}
impl CharacterCommandTargetList {
    fn get_entities(&self, world: &mut World) -> Vec<Entity> {
        match self {
            CharacterCommandTargetList::Single(target) => match target {
                CharacterCommandTarget::Uuid(uuid) => {
                    let mut query = world.query::<(Entity, &CommandPickable)>();
                    query
                        .iter(world)
                        .filter(|(_, u)| u.0 == *uuid)
                        .map(|(e, _)| e)
                        .collect()
                }
                CharacterCommandTarget::Identifier(id) => match id {
                    CharacterCommandIdentifier::Player => {
                        let mut query =
                            world.query_filtered::<Entity, (With<CommandPickable>, With<Player>)>();
                        vec![query.single(world).expect("Failed to get player!")]
                    }
                },
            },
        }
    }
}

#[derive(Debug)]
enum CharacterCommandTarget {
    Uuid(Uuid),
    Identifier(CharacterCommandIdentifier),
}

#[derive(Debug)]
enum CharacterCommandIdentifier {
    Player,
}

fn parse_operation(input: &mut &str) -> ModalResult<CharacterOperation> {
    info!("Parsing operation from: '{}'", input);
    alt((
        ("kill", ()).map(|_| CharacterOperation::Kill),
        ("attribute", preceded(space1, parse_attribute))
            .map(|(_, attr)| CharacterOperation::Attribute(attr))
            .context(StrContext::Label("attribute <attribute>")),
        ("ai", preceded(space1, parse_ai_operation))
            .map(|(_, operation)| CharacterOperation::Ai(operation))
            .context(StrContext::Label("ai <operation>")),
        fail.context(StrContext::Label("operation"))
            .context(StrContext::Expected(StrContextValue::StringLiteral("kill")))
            .context(StrContext::Expected(StrContextValue::StringLiteral(
                "attribute",
            )))
            .context(StrContext::Expected(StrContextValue::StringLiteral("ai"))),
    ))
    .parse_next(input)
}

#[derive(Debug, Display)]
enum CharacterOperation {
    Kill,
    Attribute(Attribute),
    Ai(AiOperation),
}

fn parse_attribute(input: &mut &str) -> ModalResult<Attribute> {
    alt(((
        "health",
        preceded(
            space1,
            (
                parse_attribute_operation.context(StrContext::Label("operation")),
                preceded(space1, parse_digits::<u32>).context(StrContext::Label("amount")),
            ),
        ),
    )
        .map(|(_, (operation, amount))| Attribute::Health { operation, amount }),))
    .parse_next(input)
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

fn parse_ai_operation(input: &mut &str) -> ModalResult<AiOperation> {
    alt((
        ("enable", ()).map(|_| AiOperation::Enable),
        ("disable", ()).map(|_| AiOperation::Disable),
        ("pathfinder", preceded(space1, parse_pathfinder_mode))
            .map(|(_, mode)| AiOperation::Pathfinder(mode)),
    ))
    .context(StrContext::Label("ai"))
    .parse_next(input)
}

#[derive(Debug, Clone, Copy)]
enum AiOperation {
    Enable,
    Disable,
    Pathfinder(PathfinderMode),
}

fn parse_pathfinder_mode(input: &mut &str) -> ModalResult<PathfinderMode> {
    alt((
        ("follow", ()).map(|_| PathfinderMode::Follow),
        ("wander", ()).map(|_| PathfinderMode::Wander),
    ))
    .context(StrContext::Label("pathfinder"))
    .parse_next(input)
}

#[derive(Debug, Clone, Copy)]
enum PathfinderMode {
    Follow,
    Wander,
}
