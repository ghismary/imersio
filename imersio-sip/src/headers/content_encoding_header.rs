//! SIP Content-Encoding header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{ContentEncoding, ContentEncodings};

/// Representation of a Content-Encoding header.
///
/// The Content-Encoding header field is used as a modifier to the
/// "media-type". When present, its value indicates what additional content
/// codings have been applied to the entity-body, and thus what decoding
/// mechanisms MUST be applied in order to obtain the media-type referenced
/// by the Content-Type header field. Content-Encoding is primarily used to
/// allow a body to be compressed without losing the identity of its
/// underlying media type.
///
/// If multiple encodings have been applied to an entity-body, the content
/// codings MUST be listed in the order in which they were applied.
///
/// [[RFC3261, Section 20.12](https://datatracker.ietf.org/doc/html/rfc3261#section-20.12)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct ContentEncodingHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    encodings: ContentEncodings,
}

impl ContentEncodingHeader {
    pub(crate) fn new(header: GenericHeader, encodings: Vec<ContentEncoding>) -> Self {
        Self {
            header,
            encodings: encodings.into(),
        }
    }

    /// Get a reference to the encodings from the Content-Encoding header.
    pub fn encodings(&self) -> &ContentEncodings {
        &self.encodings
    }
}

impl HeaderAccessor for ContentEncodingHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("e")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Content-Encoding")
    }
    fn normalized_value(&self) -> String {
        self.encodings.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::content_encoding::parser::content_coding;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{ContentEncodingHeader, Header, TokenString};
    use nom::{
        branch::alt,
        bytes::complete::{tag, tag_no_case},
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn content_encoding(input: &str) -> ParserResult<&str, Header> {
        context(
            "Content-Encoding header",
            map(
                tuple((
                    map(
                        alt((tag_no_case("Content-Encoding"), tag("e"))),
                        TokenString::new,
                    ),
                    hcolon,
                    cut(consumed(separated_list1(comma, content_coding))),
                )),
                |(name, separator, (value, encodings))| {
                    Header::ContentEncoding(ContentEncodingHeader::new(
                        GenericHeader::new(name, separator, value),
                        encodings,
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
        ContentEncodingHeader, Header,
    };
    use claims::assert_ok;

    valid_header!(ContentEncoding, ContentEncodingHeader, "Content-Encoding");
    header_equality!(ContentEncoding, "Content-Encoding");
    header_inequality!(ContentEncoding, "Content-Encoding");

    #[test]
    fn test_valid_content_encoding_header() {
        valid_header("Content-Encoding: gzip", |header| {
            assert_eq!(header.encodings().len(), 1);
            assert_eq!(header.encodings().first().unwrap(), "gzip");
        });
    }

    #[test]
    fn test_valid_content_encoding_header_in_compact_form() {
        valid_header("e: tar", |header| {
            assert_eq!(header.encodings().len(), 1);
            assert_eq!(header.encodings().first().unwrap(), "tar");
        });
    }

    #[test]
    fn test_invalid_content_encoding_header_empty() {
        invalid_header("Content-Encoding:");
    }

    #[test]
    fn test_invalid_content_encoding_header_empty_with_space_characters() {
        invalid_header("Content-Encoding:    ");
    }

    #[test]
    fn test_invalid_content_encoding_header_with_invalid_character() {
        invalid_header("Content-Encoding: 😁");
    }

    #[test]
    fn test_content_encoding_header_equality_same_header_with_space_characters_differences() {
        header_equality("Content-Encoding: gzip", "Content-Encoding:  gzip");
    }

    #[test]
    fn test_content_encoding_header_equality_same_encodings_in_a_different_order() {
        header_equality("Content-Encoding: gzip, tar", "Content-Encoding: tar, gzip");
    }

    #[test]
    fn test_content_encoding_header_equality_same_encodings_with_different_cases() {
        header_equality("Content-Encoding: gzip", "content-encoding: GZIP");
    }

    #[test]
    fn test_content_encoding_header_inequality_with_different_encodings() {
        header_inequality("Content-Encoding: gzip", "Content-Encoding: tar");
    }

    #[test]
    fn test_content_encoding_header_inequality_with_first_having_more_encodings_than_the_second() {
        header_inequality("Content-Encoding: gzip, tar", "Content-Encoding: tar");
    }

    #[test]
    fn test_content_encoding_header_inequality_with_first_having_less_encodings_than_the_second() {
        header_inequality("Content-Encoding: gzip", "Content-Encoding: tar, gzip");
    }

    #[test]
    fn test_content_encoding_header_to_string() {
        let header = Header::try_from("content-enCoding:  tar , GZIP");
        if let Header::ContentEncoding(header) = header.unwrap() {
            assert_eq!(header.to_string(), "content-enCoding:  tar , GZIP");
            assert_eq!(header.to_normalized_string(), "Content-Encoding: tar, gzip");
            assert_eq!(header.to_compact_string(), "e: tar, gzip");
        }
    }
}
