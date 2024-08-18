//! SIP Proxy-Require header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{OptionTag, OptionTags};

/// Representation of a Proxy-Require header.
///
/// The Proxy-Require header field is used to indicate proxy-sensitive features that must be
/// supported by the proxy.
///
/// [[RFC3261, Section 20.29](https://datatracker.ietf.org/doc/html/rfc3261#section-20.29)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct ProxyRequireHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    option_tags: OptionTags,
}

impl ProxyRequireHeader {
    pub(crate) fn new(header: GenericHeader, option_tags: Vec<OptionTag>) -> Self {
        Self {
            header,
            option_tags: option_tags.into(),
        }
    }

    /// Get a reference to the option tags from the Proxy-Require header.
    pub fn option_tags(&self) -> &OptionTags {
        &self.option_tags
    }
}

impl HeaderAccessor for ProxyRequireHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Proxy-Require")
    }
    fn normalized_value(&self) -> String {
        self.option_tags.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::option_tag::parser::option_tag;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{Header, ProxyRequireHeader, TokenString};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn proxy_require(input: &str) -> ParserResult<&str, Header> {
        context(
            "Proxy-Require header",
            map(
                tuple((
                    map(tag_no_case("Proxy-Require"), TokenString::new),
                    hcolon,
                    cut(consumed(separated_list1(comma, option_tag))),
                )),
                |(name, separator, (value, tags))| {
                    Header::ProxyRequire(ProxyRequireHeader::new(
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
        Header, ProxyRequireHeader,
    };
    use claims::assert_ok;

    valid_header!(ProxyRequire, ProxyRequireHeader, "Proxy-Require");
    header_equality!(ProxyRequire, "Proxy-Require");
    header_inequality!(ProxyRequire, "Proxy-Require");

    #[test]
    fn test_valid_proxy_require_header() {
        valid_header("Proxy-Require: foo", |header| {
            assert_eq!(header.option_tags().len(), 1);
            assert_eq!(header.option_tags().first().unwrap(), "foo");
        });
    }

    #[test]
    fn test_valid_proxy_require_header_with_several_values() {
        valid_header("Proxy-Require: foo, bar", |header| {
            assert_eq!(header.option_tags().len(), 2);
            assert_eq!(header.option_tags().first().unwrap(), "foo");
            assert_eq!(header.option_tags().last().unwrap(), "bar");
        });
    }

    #[test]
    fn test_invalid_proxy_require_header_empty() {
        invalid_header("Proxy-Require:");
    }

    #[test]
    fn test_invalid_proxy_require_header_empty_with_space_characters() {
        invalid_header("Proxy-Require:    ");
    }

    #[test]
    fn test_invalid_proxy_require_header_with_invalid_character() {
        invalid_header("Proxy-Require: 😁");
    }

    #[test]
    fn test_proxy_require_header_equality_same_header_with_space_characters_differences() {
        header_equality("Proxy-Require: foo", "Proxy-Require:  foo");
    }

    #[test]
    fn test_proxy_require_header_equality_same_encodings_in_a_different_order() {
        header_equality("Proxy-Require: foo, bar", "Proxy-Require: bar, foo");
    }

    #[test]
    fn test_proxy_require_header_equality_same_encodings_with_different_cases() {
        header_equality("Proxy-Require: foo", "proxy-require: FOO");
    }

    #[test]
    fn test_proxy_require_header_inequality_with_different_option_tags() {
        header_inequality("Proxy-Require: foo", "Proxy-Require: bar");
    }

    #[test]
    fn test_proxy_require_header_inequality_with_first_having_more_option_tags_than_the_second() {
        header_inequality("Proxy-Require: foo, bar", "Proxy-Require: bar");
    }

    #[test]
    fn test_proxy_require_header_inequality_with_first_having_less_option_tags_than_the_second() {
        header_inequality("Proxy-Require: foo", "Proxy-Require: bar, foo");
    }

    #[test]
    fn test_proxy_require_header_to_string() {
        let header = Header::try_from("proxy-reQuire:  bar , FOO");
        if let Header::ProxyRequire(header) = header.unwrap() {
            assert_eq!(header.to_string(), "proxy-reQuire:  bar , FOO");
            assert_eq!(header.to_normalized_string(), "Proxy-Require: bar, foo");
            assert_eq!(header.to_compact_string(), "Proxy-Require: bar, foo");
        }
    }
}
