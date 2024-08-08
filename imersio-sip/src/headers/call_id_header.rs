//! SIP Call-ID header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::CallId;

/// Representation of a Call-ID header.
///
/// The Call-ID header field uniquely identifies a particular invitation or
/// all registrations of a particular client.
///
/// [[RFC3261, Section 20.8](https://datatracker.ietf.org/doc/html/rfc3261#section-20.8)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct CallIdHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    call_id: CallId,
}

impl CallIdHeader {
    pub(crate) fn new(header: GenericHeader, call_id: CallId) -> Self {
        Self { header, call_id }
    }

    /// Get the call ID from the Call-ID header.
    pub fn call_id(&self) -> &str {
        self.call_id.as_ref()
    }
}

impl HeaderAccessor for CallIdHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("i")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Call-ID")
    }
    fn normalized_value(&self) -> String {
        self.call_id.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::call_id::parser::callid;
    use crate::headers::GenericHeader;
    use crate::parser::{hcolon, ParserResult};
    use crate::{CallIdHeader, Header};
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        sequence::tuple,
    };

    pub(crate) fn call_id(input: &str) -> ParserResult<&str, Header> {
        context(
            "Call-ID header",
            map(
                tuple((
                    alt((tag_no_case("Call-ID"), tag_no_case("i"))),
                    hcolon,
                    cut(consumed(callid)),
                )),
                |(name, separator, (value, call_id))| {
                    Header::CallId(CallIdHeader::new(
                        GenericHeader::new(name, separator, value),
                        call_id,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        CallIdHeader, Header,
    };
    use claims::assert_ok;

    valid_header!(CallId, CallIdHeader, "Call-ID");
    header_equality!(CallId, "Call-ID");
    header_inequality!(CallId, "Call-ID");

    #[test]
    fn test_valid_call_id_header_with_at_character() {
        valid_header(
            "Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com",
            |header| {
                assert_eq!(
                    header.call_id(),
                    "f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com"
                );
            },
        );
    }

    #[test]
    fn test_valid_call_id_header_without_at_character() {
        valid_header("Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6", |header| {
            assert_eq!(header.call_id(), "f81d4fae-7dec-11d0-a765-00a0c91e6bf6");
        });
    }

    #[test]
    fn test_invalid_call_id_header_empty() {
        invalid_header("Call-ID:");
    }

    #[test]
    fn test_invalid_call_id_header_empty_with_space_characters() {
        invalid_header("Call-ID:    ");
    }

    #[test]
    fn test_invalid_call_id_header_with_invalid_character() {
        invalid_header("Call-ID: 😁");
    }

    #[test]
    fn test_call_id_header_equality_same_header_with_space_characters_differences() {
        header_equality("Call-ID: a84b4c76e66710", "Call-ID:  a84b4c76e66710");
    }

    #[test]
    fn test_call_id_header_inequality_different_values() {
        header_inequality(
            "Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com",
            "Call-ID: a84b4c76e66710",
        );
    }

    #[test]
    fn test_call_id_header_inequality_one_with_at_part_the_other_without() {
        header_inequality(
            "Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com",
            "Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
        );
    }

    #[test]
    fn test_call_id_header_inequality_same_value_with_different_cases() {
        header_inequality("Call-ID: a84b4c76e66710", "Call-ID: A84B4C76E66710");
    }

    #[test]
    fn test_call_id_header_to_string() {
        let header =
            Header::try_from("CalL-iD  :     f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com");
        if let Header::CallId(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "CalL-iD  :     f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com"
            );
            assert_eq!(
                header.to_compact_string(),
                "i: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com"
            );
        }
    }
}