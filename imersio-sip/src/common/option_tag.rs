use derive_more::Display;
use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::Error;

/// Representation of the list of option tags in a `Proxy-Require` header.
///
/// This is usable as an iterator.
pub type OptionTags = HeaderValueCollection<OptionTag>;

/// Representation of an option tag contained in a `Proxy-Require` header.
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
        crate::header::parser::option_tag(value.as_bytes())
            .map(|(_, tag)| tag)
            .map_err(|_| Error::InvalidOptionTag(value.to_string()))
    }
}
