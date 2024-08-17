//! SIP Supported header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{OptionTag, OptionTags};

/// Representation of a Supported header.
///
/// The Supported header field enumerates all the extensions supported by the UAC or UAS.
///
/// [[RFC3261, Section 20.37](https://datatracker.ietf.org/doc/html/rfc3261#section-20.37)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct SupportedHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    option_tags: OptionTags,
}

impl SupportedHeader {
    pub(crate) fn new(header: GenericHeader, option_tags: Vec<OptionTag>) -> Self {
        Self {
            header,
            option_tags: option_tags.into(),
        }
    }

    /// Get a reference to the option tags from the Supported header.
    pub fn option_tags(&self) -> &OptionTags {
        &self.option_tags
    }
}

impl HeaderAccessor for SupportedHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("k")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Supported")
    }
    fn normalized_value(&self) -> String {
        self.option_tags.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::option_tag::parser::option_tag;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{Header, SupportedHeader};
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn supported(input: &str) -> ParserResult<&str, Header> {
        context(
            "Supported header",
            map(
                tuple((
                    alt((tag_no_case("Supported"), tag_no_case("k"))),
                    hcolon,
                    cut(consumed(separated_list1(comma, option_tag))),
                )),
                |(name, separator, (value, tags))| {
                    Header::Supported(SupportedHeader::new(
                        GenericHeader::new(name, separator, value),
                        tags,
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
        Header, SupportedHeader,
    };
    use claims::assert_ok;

    valid_header!(Supported, SupportedHeader, "Supported");
    header_equality!(Supported, "Supported");
    header_inequality!(Supported, "Supported");

    #[test]
    fn test_valid_supported_header() {
        valid_header("Supported: 100rel", |header| {
            assert_eq!(header.option_tags().len(), 1);
            assert_eq!(header.option_tags().first().unwrap(), "100rel");
        });
    }

    #[test]
    fn test_valid_supported_header_with_several_values() {
        valid_header("Supported: 100rel, other", |header| {
            assert_eq!(header.option_tags().len(), 2);
            assert_eq!(header.option_tags().first().unwrap(), "100rel");
            assert_eq!(header.option_tags().last().unwrap(), "other");
        });
    }

    #[test]
    fn test_valid_supported_header_in_compact_form() {
        valid_header("k: 100rel", |header| {
            assert_eq!(header.option_tags().len(), 1);
            assert_eq!(header.option_tags().first().unwrap(), "100rel");
        });
    }

    #[test]
    fn test_invalid_supported_header_empty() {
        invalid_header("Supported:");
    }

    #[test]
    fn test_invalid_supported_header_empty_with_space_characters() {
        invalid_header("Supported:    ");
    }

    #[test]
    fn test_invalid_supported_header_with_invalid_character() {
        invalid_header("Supported: 😁");
    }

    #[test]
    fn test_supported_header_equality_same_header_with_space_characters_differences() {
        header_equality("Supported: 100rel", "Supported:  100rel");
    }

    #[test]
    fn test_supported_header_equality_same_encodings_in_a_different_order() {
        header_equality("Supported: 100rel, other", "Supported: other, 100rel");
    }

    #[test]
    fn test_supported_header_equality_same_encodings_with_different_cases() {
        header_equality("Supported: 100rel", "supported: 100REL");
    }

    #[test]
    fn test_supported_header_inequality_with_different_option_tags() {
        header_inequality("Supported: 100rel", "Supported: other");
    }

    #[test]
    fn test_supported_header_inequality_with_first_having_more_option_tags_than_the_second() {
        header_inequality("Supported: 100rel, other", "Supported: other");
    }

    #[test]
    fn test_supported_header_inequality_with_first_having_less_option_tags_than_the_second() {
        header_inequality("Supported: 100rel", "Supported: other, 100rel");
    }

    #[test]
    fn test_supported_header_to_string() {
        let header = Header::try_from("suPPorteD:  other , 100REL");
        if let Header::Supported(header) = header.unwrap() {
            assert_eq!(header.to_string(), "suPPorteD:  other , 100REL");
            assert_eq!(header.to_normalized_string(), "Supported: other, 100rel");
            assert_eq!(header.to_compact_string(), "k: other, 100rel");
        }
    }
}
