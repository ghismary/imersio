use std::{hash::Hash, str::FromStr};

use partial_eq_refs::PartialEqRefs;

use crate::Error;

#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ContentEncoding(String);

impl ContentEncoding {
    pub(crate) fn new<S: Into<String>>(encoding: S) -> Self {
        Self(encoding.into())
    }
}

impl std::fmt::Display for ContentEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_ascii_lowercase())
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

impl AsRef<str> for ContentEncoding {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for ContentEncoding {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::header::parser::content_coding(s.as_bytes())
            .map(|(_, encoding)| encoding)
            .map_err(|_| Error::InvalidContentEncoding(s.to_string()))
    }
}

impl Hash for ContentEncoding {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state)
    }
}
