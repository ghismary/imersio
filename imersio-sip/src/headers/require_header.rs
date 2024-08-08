//! SIP Require header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{OptionTag, OptionTags};

/// Representation of a Require header.
///
/// The Require header field is used by UACs to tell UASs about options that the UAC expects the
/// UAS to support in order to process the request. Although an optional header field, the Require
/// MUST NOT be ignored if it is present.
///
/// [[RFC3261, Section 20.32](https://datatracker.ietf.org/doc/html/rfc3261#section-20.32)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct RequireHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    option_tags: OptionTags,
}

impl RequireHeader {
    pub(crate) fn new(header: GenericHeader, option_tags: Vec<OptionTag>) -> Self {
        Self {
            header,
            option_tags: option_tags.into(),
        }
    }

    /// Get a reference to the option tags from the Require header.
    pub fn option_tags(&self) -> &OptionTags {
        &self.option_tags
    }
}

impl HeaderAccessor for RequireHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Require")
    }
    fn normalized_value(&self) -> String {
        self.option_tags.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::option_tag::parser::option_tag;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{Header, RequireHeader};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn require(input: &str) -> ParserResult<&str, Header> {
        context(
            "Require header",
            map(
                tuple((
                    tag_no_case("Require"),
                    hcolon,
                    cut(consumed(separated_list1(comma, option_tag))),
                )),
                |(name, separator, (value, tags))| {
                    Header::Require(RequireHeader::new(
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
        Header, RequireHeader,
    };
    use claims::assert_ok;

    valid_header!(Require, RequireHeader, "Require");
    header_equality!(Require, "Require");
    header_inequality!(Require, "Require");

    #[test]
    fn test_valid_require_header() {
        valid_header("Require: 100rel", |header| {
            assert_eq!(header.option_tags().len(), 1);
            assert_eq!(header.option_tags().first().unwrap(), "100rel");
        });
    }

    #[test]
    fn test_valid_require_header_with_several_values() {
        valid_header("Require: 100rel, other", |header| {
            assert_eq!(header.option_tags().len(), 2);
            assert_eq!(header.option_tags().first().unwrap(), "100rel");
            assert_eq!(header.option_tags().last().unwrap(), "other");
        });
    }

    #[test]
    fn test_invalid_require_header_empty() {
        invalid_header("Require:");
    }

    #[test]
    fn test_invalid_require_header_empty_with_space_characters() {
        invalid_header("Require:    ");
    }

    #[test]
    fn test_invalid_require_header_with_invalid_character() {
        invalid_header("Require: 😁");
    }

    #[test]
    fn test_require_header_equality_same_header_with_space_characters_differences() {
        header_equality("Require: 100rel", "Require:  100rel");
    }

    #[test]
    fn test_require_header_equality_same_encodings_in_a_different_order() {
        header_equality("Require: 100rel, other", "Require: other, 100rel");
    }

    #[test]
    fn test_require_header_equality_same_encodings_with_different_cases() {
        header_equality("Require: 100rel", "require: 100REL");
    }

    #[test]
    fn test_require_header_inequality_with_different_option_tags() {
        header_inequality("Require: 100rel", "Require: other");
    }

    #[test]
    fn test_require_header_inequality_with_first_having_more_option_tags_than_the_second() {
        header_inequality("Require: 100rel, other", "Require: other");
    }

    #[test]
    fn test_require_header_inequality_with_first_having_less_option_tags_than_the_second() {
        header_inequality("Require: 100rel", "Require: other, 100rel");
    }

    #[test]
    fn test_require_header_to_string() {
        let header = Header::try_from("reQuire:  other , 100REL");
        if let Header::Require(header) = header.unwrap() {
            assert_eq!(header.to_string(), "reQuire:  other , 100REL");
            assert_eq!(header.to_normalized_string(), "Require: other, 100rel");
            assert_eq!(header.to_compact_string(), "Require: other, 100rel");
        }
    }
}
