use bevy::asset::uuid::Uuid;
use std::str::FromStr;
use winnow::ascii::{digit1, space0};
use winnow::combinator::fail;
use winnow::error::StrContext;
use winnow::prelude::*;
use winnow::token::literal;

pub fn parse_prefix(
    expected: &'static str,
) -> impl for<'s> FnMut(&mut &'s str) -> ModalResult<&'s str> {
    move |input| {
        (
            literal(expected),  // Parse the provided prefix
            space0              // Consume any trailing whitespace
        ).parse_next(input)?;
        Ok(input)
    }
}

pub fn parse_digits<T: FromStr>(input: &mut &str) -> ModalResult<T> {
    digit1
        .parse_to()
        .context(StrContext::Label("Digit Parsing"))
        .parse_next(input)
}

pub fn parse_uuid(input: &mut &str) -> ModalResult<Uuid> {
    let uuid = take_until_whitespace(input);
    space0.parse_next(input)?;

    Uuid::parse_str(uuid).map_err(|_| {
        fail::<&str, Uuid, winnow::error::ErrMode<winnow::error::ContextError>>
            .context(StrContext::Label("Uuid Parsing"))
            .parse_next(input)
            .unwrap_err()
    })
}

pub fn take_until_whitespace<'a>(input: &mut &'a str) -> &'a str {
    match input.char_indices().find(|(_, c)| c.is_whitespace()) {
        Some((idx, c)) => {
            let before = &input[..idx];
            let after = &input[idx + c.len_utf8()..];

            *input = after;

            before
        }
        None => {
            let before = *input;
            *input = "";
            before
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_digits() {
        // GIVEN
        // A set of digits
        let mut input = "12345";

        // WHEN
        // I try to parse it
        let result = parse_digits::<u32>(&mut input);

        // THEN
        // It should parse correctly
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12345);
    }

    #[test]
    fn test_parse_digits_non_numeric() {
        // GIVEN
        // A set non-numeric characters
        let mut input = "abcde";

        // WHEN
        // I try to parse it
        let result = parse_digits::<u32>(&mut input);

        // THEN
        // It should error
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_uuid() {
        // GIVEN
        // A valid Uuid
        let uuid = Uuid::new_v4().to_string();

        // WHEN
        // I try to parse it
        let result = parse_uuid(&mut uuid.as_str());

        // THEN
        // It should parse correctly
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), uuid);
    }

    #[test]
    fn test_parse_uuid_invalid() {
        // GIVEN
        // A valid Uuid
        let mut not_uuid = "not-a-uuid";

        // WHEN
        // I try to parse it
        let result = parse_uuid(&mut not_uuid);

        // THEN
        // It should error
        assert!(result.is_err());
    }
}
