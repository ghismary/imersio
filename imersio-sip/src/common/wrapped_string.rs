use std::cmp::Ordering;
use std::{hash::Hash, ops::Deref};

/// Representation of a wrapped string, for the moment, either quoted or not wrapped.
///
/// This may get extended later on.
#[non_exhaustive]
#[derive(Clone, Debug, Eq)]
pub enum WrappedString {
    Quoted(String),
    NotWrapped(String),
}

impl WrappedString {
    pub(crate) fn new_quoted<S: Into<String>>(value: S) -> Self {
        Self::Quoted(value.into())
    }

    pub(crate) fn new_not_wrapped<S: Into<String>>(value: S) -> Self {
        Self::NotWrapped(value.into())
    }

    /// Get the value stored inside the wrapped string.
    pub fn value(&self) -> String {
        match self {
            Self::Quoted(value) => value.clone(),
            Self::NotWrapped(value) => value.to_ascii_lowercase(),
        }
    }
}

impl std::fmt::Display for WrappedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quoted(value) => write!(f, r#""{}""#, value),
            Self::NotWrapped(value) => write!(f, "{}", value),
        }
    }
}

impl PartialEq for WrappedString {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Quoted(a), Self::Quoted(b)) => a == b,
            (Self::NotWrapped(a), Self::NotWrapped(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialOrd for WrappedString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WrappedString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

impl Hash for WrappedString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Quoted(value) => format!(r#""{}""#, value).hash(state),
            Self::NotWrapped(value) => value.to_ascii_lowercase().hash(state),
        }
    }
}

impl Deref for WrappedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Quoted(value) => value,
            Self::NotWrapped(value) => value,
        }
    }
}
