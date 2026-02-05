use nom_language::error::convert_error;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::{SipError, TokenString};

/// Representation of the list of option tags in a `Proxy-Require`, `Require`, `Supported` or
/// `Unsupported` header.
///
/// This is usable as an iterator.
pub type OptionTags = ValueCollection<OptionTag>;

/// Representation of an option tag contained in a `Proxy-Require`, `Require`, `Supported` or
/// `Unsupported` header.
#[derive(Clone, Debug, Eq, derive_more::Display)]
#[display("{}", self.0.to_ascii_lowercase())]
pub struct OptionTag(TokenString);

impl OptionTag {
    pub(crate) fn new(tag: TokenString) -> Self {
        Self(tag)
    }

    /// Get the value of the option tag.
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl PartialEq for OptionTag {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<str> for OptionTag {
    fn eq(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<OptionTag> for str {
    fn eq(&self, other: &OptionTag) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<&str> for OptionTag {
    fn eq(&self, other: &&str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<OptionTag> for &str {
    fn eq(&self, other: &OptionTag) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialOrd for OptionTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OptionTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .to_ascii_lowercase()
            .cmp(&other.0.to_ascii_lowercase())
    }
}

impl Hash for OptionTag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state)
    }
}

impl AsRef<str> for OptionTag {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for OptionTag {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::option_tag(value) {
            Ok((rest, tag)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(tag)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidOptionTag(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidOptionTag(format!(
                "Incomplete option tag `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use nom::{Parser, combinator::map, error::context};

    use crate::{
        OptionTag,
        parser::{ParserResult, token},
    };

    pub(crate) fn option_tag(input: &str) -> ParserResult<&str, OptionTag> {
        context("option_tag", map(token, OptionTag::new)).parse(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_option_tag_eq() {
        assert_eq!(OptionTag::try_from("foo").unwrap(), "foo");
    }

    #[test]
    fn test_option_tag_eq_different_case() {
        assert_eq!(OptionTag::try_from("foo").unwrap(), "FOO");
    }

    #[test]
    fn test_valid_option_tag() {
        assert_ok!(OptionTag::try_from("foo"));
    }

    #[test]
    fn test_invalid_option_tag_empty() {
        assert_err!(OptionTag::try_from(""));
    }

    #[test]
    fn test_invalid_option_tag_with_invalid_character() {
        assert_err!(OptionTag::try_from("😁"));
    }

    #[test]
    fn test_valid_option_tag_with_remaining_data() {
        assert!(
            OptionTag::try_from("foo anything")
                .is_err_and(|e| e == SipError::RemainingUnparsedData(" anything".to_string()))
        );
    }
}
