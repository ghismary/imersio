//! SIP Proxy-Require header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::{header::GenericHeader, HeaderAccessor};
use crate::{OptionTag, OptionTags};

/// Representation of a Proxy-Require header.
///
/// The Proxy-Require header field is used to indicate proxy-sensitive features that must be
/// supported by the proxy.
///
/// [[RFC3261, Section 20.29](https://datatracker.ietf.org/doc/html/rfc3261#section-20.29)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
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
    crate::header::generic_header_accessors!(header);

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

#[cfg(test)]
mod tests {
    use claims::assert_ok;

    use super::ProxyRequireHeader;
    use crate::{
        header::tests::{header_equality, header_inequality, invalid_header, valid_header},
        Header, HeaderAccessor,
    };

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
