use bevy::prelude::Entity;
use winnow::ascii::digit1;
use winnow::combinator::fail;
use winnow::error::{StrContext, StrContextValue};
use winnow::prelude::*;
use winnow::token::literal;

pub fn parse_prefix(
    expected: &'static str,
) -> impl for<'s> FnMut(&mut &'s str) -> ModalResult<&'s str> {
    move |input| {
        literal(expected).parse_next(input)?;
        Ok(input)
    }
}

pub fn parse_digits(input: &mut &str) -> ModalResult<u64> {
    digit1.parse_to().context(StrContext::Label("Digit Parsing")).parse_next(input)
}

pub fn parse_entity(input: &mut &str) -> ModalResult<Entity> {
    let bits = parse_digits(input)?;
    Entity::try_from_bits(bits).ok_or(
        fail.context(StrContext::Label("Entity Id"))
            .context(StrContext::Expected(StrContextValue::Description(
                "Failed to parse Entity from bits",
            )))
            .parse_next(input)?,
    )
}
