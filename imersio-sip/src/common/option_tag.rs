use derive_more::Display;
use nom::error::convert_error;
use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::Error;

/// Representation of the list of option tags in a `Proxy-Require` or in a `Require` header.
///
/// This is usable as an iterator.
pub type OptionTags = HeaderValueCollection<OptionTag>;

/// Representation of an option tag contained in a `Proxy-Require` or in a `Require` header.
#[derive(Clone, Debug, Display, Eq, PartialEqRefs)]
#[display(fmt = "{}", "self.0.to_ascii_lowercase()")]
pub struct OptionTag(String);

impl OptionTag {
    pub(crate) fn new<S: Into<String>>(tag: S) -> Self {
        Self(tag.into())
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
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::option_tag(value) {
            Ok((rest, tag)) => {
                if !rest.is_empty() {
                    Err(Error::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(tag)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(Error::InvalidOptionTag(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidOptionTag(format!(
                "Incomplete option tag `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use crate::parser::{token, ParserResult};
    use crate::OptionTag;
    use nom::{combinator::map, error::context};

    pub(crate) fn option_tag(input: &str) -> ParserResult<&str, OptionTag> {
        context("option_tag", map(token, OptionTag::new))(input)
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
        assert!(OptionTag::try_from("foo anything")
            .is_err_and(|e| e == Error::RemainingUnparsedData(" anything".to_string())));
    }
}
