use std::any::Any;
use crate::commands::parser::basic_parsers::parse_prefix;
use bevy::prelude::*;
use std::collections::HashMap;
use winnow::combinator::{fail, opt};
use winnow::error::{ContextError, ErrMode};
use winnow::{ModalResult, Parser, Result};

mod basic_parsers;
mod character;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CommandRegistry>();

    app.add_plugins((character::plugin,));
}

pub trait CommandRegistrar {
    fn add_debug_command<T: DebugCommand + 'static>(&mut self);
}

impl CommandRegistrar for App {
    fn add_debug_command<T: DebugCommand + 'static>(&mut self) {
        let mut registry = self.world_mut().resource_mut::<CommandRegistry>();
        if registry.0.contains_key(T::NAME) {
            panic!("Command {} already registered", T::NAME);
        }
        registry.0.insert(
            T::NAME,
            Box::new(|input: &mut &_| T::parse(input).map(|cmd| cmd as Box<dyn DynDebugCommand>)),
        );
    }
}

#[derive(Resource, Default)]
pub struct CommandRegistry(HashMap<&'static str, Box<ParserFn>>);
type ParserFn =
    dyn for<'s> Parser<&'s str, Box<dyn DynDebugCommand>, ErrMode<ContextError>> + Send + Sync;

pub trait DebugCommand: DynDebugCommand {
    const NAME: &'static str;
    /// Parses the command from the given input
    fn parse(input: &mut &str) -> ModalResult<Box<Self>>;

    /// Invokes the command
    fn invoke(&self, world: &mut World);
}

pub trait DynDebugCommand: Send + Sync {
    /// The name of the command used to invoke it
    fn name(&self) -> &'static str;
    fn invoke(&self, world: &mut World);
    fn as_any(&self) -> &dyn Any;
}

impl<T: DebugCommand + 'static> DynDebugCommand for T {
    fn name(&self) -> &'static str {
        T::NAME
    }

    fn invoke(&self, world: &mut World) {
        <T as DebugCommand>::invoke(self, world);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_command<'s>(
    input: &mut &'s str,
    registry: &'s mut CommandRegistry, // TODO: Fix this being a mutable reference?
) -> Result<Box<dyn DynDebugCommand>> {
    let mut output = None;

    for (command, parser) in registry.0.iter_mut() {
        if let Some(mut out) = opt(parse_prefix(command)).parse_next(input)? {
            match parser.parse_next(&mut out) {
                Ok(cmd) => output = Some(cmd),
                Err(err) => {
                    return match err {
                        ErrMode::Backtrack(err) => Err(err),
                        ErrMode::Cut(err) => Err(err),
                        ErrMode::Incomplete(_) => todo!(),
                    };
                }
            }
        }
    }

    match output {
        Some(cmd) => Ok(cmd),
        None => Err(fail::<&'s str, &'s str, ContextError>
            .parse_next(input)
            .unwrap_err()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn registry() -> CommandRegistry {
        let mut registry = CommandRegistry(HashMap::new());

        registry.0.insert(
            TestCommand::NAME,
            Box::new(|input: &mut &_| {
                TestCommand::parse(input).map(|cmd| cmd as Box<dyn DynDebugCommand>)
            }),
        );

        registry
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestCommand;
    impl DebugCommand for TestCommand {
        const NAME: &'static str = "test";

        fn parse(_: &mut &str) -> ModalResult<Box<Self>> {
            Ok(Box::new(Self))
        }

        fn invoke(&self, _world: &mut World) {
            unimplemented!()
        }
    }

    #[test]
    fn test_parsing() {
        // GIVEN
        // A command registry with a test command
        let mut registry = registry();

        // WHEN
        // I attempt to parse that command
        let mut input = "test";
        let result = parse_command(&mut input, &mut registry);

        // THEN
        // It should be parsed correctly
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_any().downcast_ref::<TestCommand>(), Some(&TestCommand));
    }

    #[test]
    fn test_unknown_command() {
        // GIVEN
        // A command registry with a test command
        let mut registry = registry();

        // WHEN
        // I attempt to parse an unregistered command
        let mut input = "unknown";
        let result = parse_command(&mut input, &mut registry);

        // THEN
        // It should error
        assert!(result.is_err());
    }
}
