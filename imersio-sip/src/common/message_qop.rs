use derive_more::{Deref, From, IsVariant};
use itertools::{join, Itertools};
use std::cmp::Ordering;
use std::hash::Hash;

use crate::utils::compare_vectors;

/// Representation of the list of qop in a `Proxy-Authenticate` header.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Deref, Eq, From)]
pub struct MessageQops(Vec<MessageQop>);

impl std::fmt::Display for MessageQops {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#""{}""#, join(self.deref(), ","))
    }
}

impl PartialEq for MessageQops {
    fn eq(&self, other: &Self) -> bool {
        compare_vectors(self.deref(), other.deref())
    }
}

impl Hash for MessageQops {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.iter().sorted().for_each(|value| value.hash(state))
    }
}

/// Representation of a Qop parameter value.
#[derive(Clone, Debug, Eq, IsVariant)]
pub enum MessageQop {
    /// auth qop.
    Auth,
    /// auth-int qop.
    AuthInt,
    /// Any other qop value.
    Other(String),
}

impl MessageQop {
    pub(crate) fn new<S: Into<String>>(qop: S) -> Self {
        let qop: String = qop.into();
        match qop.to_ascii_lowercase().as_str() {
            "auth" => Self::Auth,
            "auth-int" => Self::AuthInt,
            _ => Self::Other(qop),
        }
    }

    /// Get the value of the qop.
    pub fn value(&self) -> &str {
        match self {
            Self::Auth => "auth",
            Self::AuthInt => "auth-int",
            Self::Other(value) => value,
        }
    }
}

impl std::fmt::Display for MessageQop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq for MessageQop {
    fn eq(&self, other: &MessageQop) -> bool {
        match (self, other) {
            (Self::Auth, Self::Auth) | (Self::AuthInt, Self::AuthInt) => true,
            (Self::Other(a), Self::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialOrd for MessageQop {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MessageQop {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(other.value())
    }
}

impl Hash for MessageQop {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}

impl From<&str> for MessageQop {
    fn from(value: &str) -> Self {
        MessageQop::new(value)
    }
}
