//! SIP Max-Forwards header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};

/// Representation of a Max-Forwards header.
///
/// The Max-Forwards header field must be used with any SIP method to limit the number of proxies
/// or gateways that can forward the request to the next downstream server. This can also be useful
/// when the client is attempting to trace a request chain that appears to be failing or looping
/// in mid-chain.
///
/// [[RFC3261, Section 20.22](https://datatracker.ietf.org/doc/html/rfc3261#section-20.22)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
pub struct MaxForwardsHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    max_forwards: u8,
}

impl MaxForwardsHeader {
    pub(crate) fn new(header: GenericHeader, max_forwards: u8) -> Self {
        Self {
            header,
            max_forwards,
        }
    }

    /// Get the max forwards from the Max-Forwards header.
    pub fn max_forwards(&self) -> u8 {
        self.max_forwards
    }
}

impl HeaderAccessor for MaxForwardsHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Max-Forwards")
    }
    fn normalized_value(&self) -> String {
        self.max_forwards.to_string()
    }
}

pub(crate) mod parser {
    use nom::{
        Parser,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map, recognize},
        error::context,
        multi::many1,
    };

    use crate::{
        Header, MaxForwardsHeader, TokenString,
        headers::GenericHeader,
        parser::{ParserResult, digit, hcolon},
    };

    pub(crate) fn max_forwards(input: &str) -> ParserResult<&str, Header> {
        context(
            "Max-Forwards header",
            map(
                (
                    map(tag_no_case("Max-Forwards"), TokenString::new),
                    hcolon,
                    cut(consumed(map(recognize(many1(digit)), |value| {
                        value.parse::<u8>().unwrap_or(u8::MAX)
                    }))),
                ),
                |(name, separator, (value, max_forwards))| {
                    Header::MaxForwards(MaxForwardsHeader::new(
                        GenericHeader::new(name, separator, value),
                        max_forwards,
                    ))
                },
            ),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Header, MaxForwardsHeader,
        headers::{
            HeaderAccessor,
            tests::{header_equality, header_inequality, invalid_header, valid_header},
        },
    };
    use claims::assert_ok;

    valid_header!(MaxForwards, MaxForwardsHeader, "Max-Forwards");
    header_equality!(MaxForwards, "Max-Forwards");
    header_inequality!(MaxForwards, "Max-Forwards");

    #[test]
    fn test_valid_max_forwards_header() {
        valid_header("Max-Forwards: 6", |header| {
            assert_eq!(header.max_forwards(), 6);
        });
    }

    #[test]
    fn test_valid_max_forwards_header_with_value_too_big() {
        valid_header("Max-Forwards: 263", |header| {
            assert_eq!(header.max_forwards(), u8::MAX);
        });
    }

    #[test]
    fn test_invalid_max_forwards_header_empty() {
        invalid_header("Max-Forwards:");
    }

    #[test]
    fn test_invalid_max_forwards_header_empty_with_space_characters() {
        invalid_header("Max-Forwards:    ");
    }

    #[test]
    fn test_invalid_max_forwards_header_with_invalid_character() {
        invalid_header("Max-Forwards: 😁");
    }

    #[test]
    fn test_max_forwards_header_equality_same_header_with_space_characters_differences() {
        header_equality("Max-Forwards: 6", "Max-Forwards :   6");
    }

    #[test]
    fn test_max_forwards_header_inequality_different_values() {
        header_inequality("Max-Forwards: 16", "Max-Forwards: 70");
    }

    #[test]
    fn test_max_forwards_header_to_string() {
        let header = Header::try_from("maX-forwardS  :     28");
        if let Header::MaxForwards(header) = header.unwrap() {
            assert_eq!(header.to_string(), "maX-forwardS  :     28");
            assert_eq!(header.to_normalized_string(), "Max-Forwards: 28");
            assert_eq!(header.to_compact_string(), "Max-Forwards: 28");
        }
    }
}
