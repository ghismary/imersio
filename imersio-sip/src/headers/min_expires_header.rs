//! SIP Min-Expires header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};

/// Representation of a Min-Expires header.
///
/// The Min-Expires header field conveys the minimum refresh interval supported for soft-state
/// elements managed by that server. This includes Contact header fields that are stored by a
/// registrar.
///
/// [[RFC3261, Section 20.23](https://datatracker.ietf.org/doc/html/rfc3261#section-20.23)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct MinExpiresHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    min_expires: u32,
}

impl MinExpiresHeader {
    pub(crate) fn new(header: GenericHeader, min_expires: u32) -> Self {
        Self {
            header,
            min_expires,
        }
    }

    /// Get the min expires from the Min-Expires header.
    pub fn min_expires(&self) -> u32 {
        self.min_expires
    }
}

impl HeaderAccessor for MinExpiresHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Min-Expires")
    }
    fn normalized_value(&self) -> String {
        self.min_expires.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::contact_parameter::parser::delta_seconds;
    use crate::headers::GenericHeader;
    use crate::parser::{hcolon, ParserResult};
    use crate::{Header, MinExpiresHeader};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        sequence::tuple,
    };

    pub(crate) fn min_expires(input: &str) -> ParserResult<&str, Header> {
        context(
            "Min-Expires header",
            map(
                tuple((
                    tag_no_case("Min-Expires"),
                    hcolon,
                    cut(consumed(delta_seconds)),
                )),
                |(name, separator, (value, min_expires))| {
                    Header::MinExpires(MinExpiresHeader::new(
                        GenericHeader::new(name, separator, value),
                        min_expires,
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
        Header, MinExpiresHeader,
    };
    use claims::assert_ok;

    valid_header!(MinExpires, MinExpiresHeader, "Min-Expires");
    header_equality!(MinExpires, "Min-Expires");
    header_inequality!(MinExpires, "Min-Expires");

    #[test]
    fn test_valid_min_expires_header() {
        valid_header("Min-Expires: 60", |header| {
            assert_eq!(header.min_expires(), 60);
        });
    }

    #[test]
    fn test_valid_min_expires_header_with_value_too_big() {
        valid_header("Min-Expires: 4294968000", |header| {
            assert_eq!(header.min_expires(), u32::MAX);
        });
    }

    #[test]
    fn test_invalid_min_expires_header_empty() {
        invalid_header("Min-Expires:");
    }

    #[test]
    fn test_invalid_min_expires_header_empty_with_space_characters() {
        invalid_header("Min-Expires:    ");
    }

    #[test]
    fn test_invalid_min_expires_header_with_invalid_character() {
        invalid_header("Min-Expires: 😁");
    }

    #[test]
    fn test_min_expires_header_equality_same_header_with_space_characters_differences() {
        header_equality("Min-Expires: 3600", "Min-Expires :   3600");
    }

    #[test]
    fn test_min_expires_header_inequality_different_values() {
        header_inequality("Min-Expires: 3600", "Min-Expires: 60");
    }

    #[test]
    fn test_min_expires_header_to_string() {
        let header = Header::try_from("mIn-eXpires  :     3600");
        if let Header::MinExpires(header) = header.unwrap() {
            assert_eq!(header.to_string(), "mIn-eXpires  :     3600");
            assert_eq!(header.to_normalized_string(), "Min-Expires: 3600");
            assert_eq!(header.to_compact_string(), "Min-Expires: 3600");
        }
    }
}
