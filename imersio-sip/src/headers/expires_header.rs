//! SIP Expires header parsing and generation.

use chrono::TimeDelta;
use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};

/// Representation of an Expires header.
///
/// The Expires header field gives the relative time after which the message (or content) expires.
/// The precise meaning of this is method dependent.
/// The expiration time in an INVITE does not affect the duration of the actual session that may
/// result from the invitation. Session description protocols may offer the ability to express time
/// limits on the session duration, however.
///
/// [[RFC3261, Section 20.19](https://datatracker.ietf.org/doc/html/rfc3261#section-20.19)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct ExpiresHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    expires: TimeDelta,
}

impl ExpiresHeader {
    pub(crate) fn new(header: GenericHeader, expires: TimeDelta) -> Self {
        Self { header, expires }
    }

    /// Get the expires from the Expires header.
    pub fn expires(&self) -> TimeDelta {
        self.expires
    }
}

impl HeaderAccessor for ExpiresHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Expires")
    }
    fn normalized_value(&self) -> String {
        self.expires.num_seconds().to_string()
    }
}

pub(crate) mod parser {
    use crate::common::contact_parameter::parser::delta_seconds;
    use crate::headers::GenericHeader;
    use crate::parser::{hcolon, ParserResult};
    use crate::{ExpiresHeader, Header, TokenString};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        sequence::tuple,
    };

    pub(crate) fn expires(input: &str) -> ParserResult<&str, Header> {
        context(
            "Expires header",
            map(
                tuple((
                    map(tag_no_case("Expires"), TokenString::new),
                    hcolon,
                    cut(consumed(delta_seconds)),
                )),
                |(name, separator, (value, expires))| {
                    Header::Expires(ExpiresHeader::new(
                        GenericHeader::new(name, separator, value),
                        expires,
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
        ExpiresHeader, Header,
    };
    use chrono::TimeDelta;
    use claims::assert_ok;

    valid_header!(Expires, ExpiresHeader, "Expires");
    header_equality!(Expires, "Expires");
    header_inequality!(Expires, "Expires");

    #[test]
    fn test_valid_expires_header() {
        valid_header("Expires: 5", |header| {
            assert_eq!(header.expires(), TimeDelta::seconds(5));
        });
    }

    #[test]
    fn test_valid_expires_header_with_value_too_big() {
        valid_header("Expires: 4294968000", |header| {
            assert_eq!(header.expires(), TimeDelta::seconds(u32::MAX as i64));
        });
    }

    #[test]
    fn test_invalid_expires_header_empty() {
        invalid_header("Expires:");
    }

    #[test]
    fn test_invalid_expires_header_empty_with_space_characters() {
        invalid_header("Expires:    ");
    }

    #[test]
    fn test_invalid_expires_header_with_invalid_character() {
        invalid_header("Expires: 😁");
    }

    #[test]
    fn test_expires_header_equality_same_header_with_space_characters_differences() {
        header_equality("Expires: 3600", "Expires :   3600");
    }

    #[test]
    fn test_expires_header_inequality_different_values() {
        header_inequality("Expires: 3600", "Expires: 5");
    }

    #[test]
    fn test_expires_header_to_string() {
        let header = Header::try_from("eXpires  :     3600");
        if let Header::Expires(header) = header.unwrap() {
            assert_eq!(header.to_string(), "eXpires  :     3600");
            assert_eq!(header.to_normalized_string(), "Expires: 3600");
            assert_eq!(header.to_compact_string(), "Expires: 3600");
        }
    }
}
