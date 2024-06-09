use derive_more::Display;
use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::Error;

#[derive(Clone, Debug, Display, Eq, PartialEqRefs)]
#[display(fmt = "{}", "self.0.to_ascii_lowercase()")]
pub struct ContentEncoding(String);

impl ContentEncoding {
    pub(crate) fn new<S: Into<String>>(encoding: S) -> Self {
        Self(encoding.into())
    }
}

impl PartialEq for ContentEncoding {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<str> for ContentEncoding {
    fn eq(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<ContentEncoding> for str {
    fn eq(&self, other: &ContentEncoding) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<&str> for ContentEncoding {
    fn eq(&self, other: &&str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<ContentEncoding> for &str {
    fn eq(&self, other: &ContentEncoding) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialOrd for ContentEncoding {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContentEncoding {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .to_ascii_lowercase()
            .cmp(&other.0.to_ascii_lowercase())
    }
}

impl Hash for ContentEncoding {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state)
    }
}

impl AsRef<str> for ContentEncoding {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ContentEncoding {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        crate::header::parser::content_coding(value.as_bytes())
            .map(|(_, encoding)| encoding)
            .map_err(|_| Error::InvalidContentEncoding(value.to_string()))
    }
}
