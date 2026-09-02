use winnow::Result;
use winnow::ascii::digit1;
use winnow::prelude::*;
use winnow::token::literal;

pub fn parse_prefix(expected: &'static str) -> impl for<'s> FnMut(&mut &'s str) -> Result<&'s str> {
    move |input| {
        let actual = literal(expected).parse_next(input)?;
        Ok(actual)
    }
}

pub fn parse_digits(input: &mut &str) -> Result<usize> {
    digit1.parse_to().parse_next(input)
}
