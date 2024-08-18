//! SIP Content-Length header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};

/// Representation of a Content-Length header.
///
/// The Content-Length header field indicates the size of the message body,
/// in decimal number of octets, sent to the recipient. Applications SHOULD
/// use this field to indicate the size of the message body to be
/// transferred, regardless of the media type of the entity. If a
/// stream-based protocol (such as TCP) is used as transport, the header field
/// MUST be used.
///
/// [[RFC3261, Section 20.14](https://datatracker.ietf.org/doc/html/rfc3261#section-20.14)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct ContentLengthHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    content_length: u32,
}

impl ContentLengthHeader {
    pub(crate) fn new(header: GenericHeader, content_length: u32) -> Self {
        Self {
            header,
            content_length,
        }
    }

    /// Get the content length from the Content-Length header.
    pub fn content_length(&self) -> u32 {
        self.content_length
    }
}

impl HeaderAccessor for ContentLengthHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("l")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Content-Length")
    }
    fn normalized_value(&self) -> String {
        format!("{}", self.content_length)
    }
}

pub(crate) mod parser {
    use crate::headers::GenericHeader;
    use crate::parser::{digit, hcolon, ParserResult};
    use crate::{ContentLengthHeader, Header, TokenString};
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map, recognize},
        error::context,
        multi::many1,
        sequence::tuple,
    };

    pub(crate) fn content_length(input: &str) -> ParserResult<&str, Header> {
        context(
            "Content-Length header",
            map(
                tuple((
                    map(
                        alt((tag_no_case("Content-Length"), tag_no_case("l"))),
                        TokenString::new,
                    ),
                    hcolon,
                    cut(consumed(map(recognize(many1(digit)), |l| {
                        l.parse::<u32>().unwrap()
                    }))),
                )),
                |(name, separator, (value, content_length))| {
                    Header::ContentLength(ContentLengthHeader::new(
                        GenericHeader::new(name, separator, value),
                        content_length,
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
        ContentLengthHeader, Header,
    };
    use claims::assert_ok;

    valid_header!(ContentLength, ContentLengthHeader, "Content-Length");
    header_equality!(ContentLength, "Content-Length");
    header_inequality!(ContentLength, "Content-Length");

    #[test]
    fn test_valid_content_length_header() {
        valid_header("Content-Length: 349", |header| {
            assert_eq!(header.content_length(), 349);
        });
    }

    #[test]
    fn test_valid_content_length_header_in_compact_form() {
        valid_header("l: 173", |header| {
            assert_eq!(header.content_length(), 173);
        });
    }

    #[test]
    fn test_invalid_content_length_header_empty() {
        invalid_header("Content-Length:");
    }

    #[test]
    fn test_invalid_content_length_header_empty_with_space_characters() {
        invalid_header("Content-Length:    ");
    }

    #[test]
    fn test_invalid_content_length_header_with_invalid_character_1() {
        invalid_header("Content-Length: 😁");
    }

    #[test]
    fn test_invalid_content_length_header_with_invalid_character_2() {
        invalid_header("Content-Length: mysize");
    }

    #[test]
    fn test_content_length_header_equality_same_header_with_space_characters_differences() {
        header_equality("Content-Length: 349", "Content-Length:     349");
    }

    #[test]
    fn test_content_length_header_equality_one_in_normal_form_the_other_in_compact_form() {
        header_equality("Content-Length: 283", "l: 283");
    }

    #[test]
    fn test_content_length_header_inequality() {
        header_inequality("Content-Length: 349", "Content-Length: 173");
    }

    #[test]
    fn test_content_length_header_to_string() {
        let header = Header::try_from("cOntEnt-lEngth  :   349");
        if let Header::ContentLength(header) = header.unwrap() {
            assert_eq!(header.to_string(), "cOntEnt-lEngth  :   349");
            assert_eq!(header.to_normalized_string(), "Content-Length: 349");
            assert_eq!(header.to_compact_string(), "l: 349");
        }
    }
}
