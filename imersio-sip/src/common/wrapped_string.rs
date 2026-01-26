use std::cmp::Ordering;
use std::{hash::Hash, ops::Deref};

/// Representation of a wrapped string, for the moment, either quoted or not wrapped.
///
/// This may get extended later on.
#[non_exhaustive]
#[derive(Clone, Debug, Eq)]
pub enum WrappedString<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    Quoted(String),
    NotWrapped(T),
}

impl<T> WrappedString<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    pub(crate) fn new_quoted<S: Into<String>>(value: S) -> Self {
        Self::Quoted(value.into())
    }

    pub(crate) fn new_not_wrapped(value: T) -> Self {
        Self::NotWrapped(value)
    }

    /// Get the value stored inside the wrapped string.
    pub fn value(&self) -> String {
        match self {
            Self::Quoted(value) => value.clone(),
            Self::NotWrapped(value) => value.as_ref().to_ascii_lowercase(),
        }
    }
}

impl<T> std::fmt::Display for WrappedString<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quoted(value) => write!(f, r#""{}""#, value),
            Self::NotWrapped(value) => write!(f, "{}", value),
        }
    }
}

impl<T> PartialEq for WrappedString<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Quoted(a), Self::Quoted(b)) => a == b,
            (Self::NotWrapped(a), Self::NotWrapped(b)) => {
                a.as_ref().eq_ignore_ascii_case(b.as_ref())
            }
            _ => false,
        }
    }
}

impl<T> PartialOrd for WrappedString<T>
where
    T: std::fmt::Display + AsRef<str> + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for WrappedString<T>
where
    T: std::fmt::Display + AsRef<str> + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

impl<T> Hash for WrappedString<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Quoted(value) => format!(r#""{}""#, value).hash(state),
            Self::NotWrapped(value) => value.as_ref().to_ascii_lowercase().hash(state),
        }
    }
}

impl<T> Deref for WrappedString<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Quoted(value) => value,
            Self::NotWrapped(value) => value.as_ref(),
        }
    }
}
