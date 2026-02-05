//! SIP Unsupported header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{OptionTag, OptionTags};

/// Representation of an Unsupported header.
///
/// The Unsupported header field lists the features not supported by the UAS.
///
/// [[RFC3261, Section 20.40](https://datatracker.ietf.org/doc/html/rfc3261#section-20.40)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
pub struct UnsupportedHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    option_tags: OptionTags,
}

impl UnsupportedHeader {
    pub(crate) fn new(header: GenericHeader, option_tags: Vec<OptionTag>) -> Self {
        Self {
            header,
            option_tags: option_tags.into(),
        }
    }

    /// Get a reference to the option tags from the Unsupported header.
    pub fn option_tags(&self) -> &OptionTags {
        &self.option_tags
    }
}

impl HeaderAccessor for UnsupportedHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Unsupported")
    }
    fn normalized_value(&self) -> String {
        self.option_tags.to_string()
    }
}

pub(crate) mod parser {
    use nom::{
        Parser,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
    };

    use crate::{
        Header, TokenString, UnsupportedHeader,
        common::option_tag::parser::option_tag,
        headers::GenericHeader,
        parser::{ParserResult, comma, hcolon},
    };

    pub(crate) fn unsupported(input: &str) -> ParserResult<&str, Header> {
        context(
            "Unsupported header",
            map(
                (
                    map(tag_no_case("Unsupported"), TokenString::new),
                    hcolon,
                    cut(consumed(separated_list1(comma, option_tag))),
                ),
                |(name, separator, (value, tags))| {
                    Header::Unsupported(UnsupportedHeader::new(
                        GenericHeader::new(name, separator, value),
                        tags,
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
        Header, UnsupportedHeader,
        headers::{
            HeaderAccessor,
            tests::{header_equality, header_inequality, invalid_header, valid_header},
        },
    };
    use claims::assert_ok;

    valid_header!(Unsupported, UnsupportedHeader, "Unsupported");
    header_equality!(Unsupported, "Unsupported");
    header_inequality!(Unsupported, "Unsupported");

    #[test]
    fn test_valid_unsupported_header() {
        valid_header("Unsupported: 100rel", |header| {
            assert_eq!(header.option_tags().len(), 1);
            assert_eq!(header.option_tags().first().unwrap(), "100rel");
        });
    }

    #[test]
    fn test_valid_unsupported_header_with_several_values() {
        valid_header("Unsupported: 100rel, other", |header| {
            assert_eq!(header.option_tags().len(), 2);
            assert_eq!(header.option_tags().first().unwrap(), "100rel");
            assert_eq!(header.option_tags().last().unwrap(), "other");
        });
    }

    #[test]
    fn test_invalid_unsupported_header_empty() {
        invalid_header("Unsupported:");
    }

    #[test]
    fn test_invalid_unsupported_header_empty_with_space_characters() {
        invalid_header("Unsupported:    ");
    }

    #[test]
    fn test_invalid_unsupported_header_with_invalid_character() {
        invalid_header("Unsupported: 😁");
    }

    #[test]
    fn test_unsupported_header_equality_same_header_with_space_characters_differences() {
        header_equality("Unsupported: 100rel", "Unsupported:  100rel");
    }

    #[test]
    fn test_unsupported_header_equality_same_encodings_in_a_different_order() {
        header_equality("Unsupported: 100rel, other", "Unsupported: other, 100rel");
    }

    #[test]
    fn test_unsupported_header_equality_same_encodings_with_different_cases() {
        header_equality("Unsupported: 100rel", "unsupported: 100REL");
    }

    #[test]
    fn test_unsupported_header_inequality_with_different_option_tags() {
        header_inequality("Unsupported: 100rel", "Unsupported: other");
    }

    #[test]
    fn test_unsupported_header_inequality_with_first_having_more_option_tags_than_the_second() {
        header_inequality("Unsupported: 100rel, other", "Unsupported: other");
    }

    #[test]
    fn test_unsupported_header_inequality_with_first_having_less_option_tags_than_the_second() {
        header_inequality("Unsupported: 100rel", "Unsupported: other, 100rel");
    }

    #[test]
    fn test_unsupported_header_to_string() {
        let header = Header::try_from("uNsuPPorteD:  other , 100REL");
        if let Header::Unsupported(header) = header.unwrap() {
            assert_eq!(header.to_string(), "uNsuPPorteD:  other , 100REL");
            assert_eq!(header.to_normalized_string(), "Unsupported: other, 100rel");
            assert_eq!(header.to_compact_string(), "Unsupported: other, 100rel");
        }
    }
}
