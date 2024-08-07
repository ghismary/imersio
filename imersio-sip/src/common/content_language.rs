use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::Error;

/// Representation of the list of languages in a `Content-Language` header.
///
/// This is usable as an iterator.
pub type ContentLanguages = HeaderValueCollection<ContentLanguage>;

/// Representation of a language contained in an `Content-Language` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ContentLanguage(String);

impl ContentLanguage {
    pub(crate) fn new<S: Into<String>>(language: S) -> Self {
        Self(language.into())
    }
}

impl std::fmt::Display for ContentLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_ascii_lowercase())
    }
}

impl PartialEq for ContentLanguage {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<str> for ContentLanguage {
    fn eq(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<ContentLanguage> for str {
    fn eq(&self, other: &ContentLanguage) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<&str> for ContentLanguage {
    fn eq(&self, other: &&str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<ContentLanguage> for &str {
    fn eq(&self, other: &ContentLanguage) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialOrd for ContentLanguage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContentLanguage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .to_ascii_lowercase()
            .cmp(&other.0.to_ascii_lowercase())
    }
}

impl Hash for ContentLanguage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state)
    }
}

impl AsRef<str> for ContentLanguage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ContentLanguage {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        crate::headers::parser::language_tag(value)
            .map(|(_, language)| language)
            .map_err(|_| Error::InvalidContentLanguage(value.to_string()))
    }
}
