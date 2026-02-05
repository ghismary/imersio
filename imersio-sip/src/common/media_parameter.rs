use std::cmp::Ordering;
use std::hash::Hash;

use crate::TokenString;
use crate::common::wrapped_string::WrappedString;

/// Representation of a media parameter.
#[derive(Clone, Debug, Eq)]
pub struct MediaParameter {
    key: TokenString,
    value: WrappedString<TokenString>,
}

impl MediaParameter {
    /// Create a `MediaParameter`.
    pub fn new(key: TokenString, value: WrappedString<TokenString>) -> Self {
        Self { key, value }
    }

    /// Get the key of the media parameter.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the value of the media parameter.
    pub fn value(&self) -> &WrappedString<TokenString> {
        &self.value
    }
}

impl std::fmt::Display for MediaParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

impl PartialEq for MediaParameter {
    fn eq(&self, other: &MediaParameter) -> bool {
        self.key().eq_ignore_ascii_case(other.key()) && self.value() == other.value()
    }
}

impl PartialOrd for MediaParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MediaParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .key()
            .to_ascii_lowercase()
            .cmp(&other.key().to_ascii_lowercase())
        {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(other.value())
    }
}

impl Hash for MediaParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        self.value().hash(state);
    }
}
