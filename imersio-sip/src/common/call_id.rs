use derive_more::Display;
use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::Error;

/// Representation of the list of call IDs in a `In-Reply-To` header.
///
/// This is usable as an iterator.
pub type CallIds = HeaderValueCollection<CallId>;

/// Representation of a call id contained in a `Call-Id` or `In-Reply-To` header.
#[derive(Clone, Debug, Display, Eq, PartialEqRefs)]
#[display(fmt = "{}", "self.0.to_ascii_lowercase()")]
pub struct CallId(String);

impl CallId {
    pub(crate) fn new<S: Into<String>>(callid: S) -> Self {
        Self(callid.into())
    }
}

impl PartialEq for CallId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<str> for CallId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<CallId> for str {
    fn eq(&self, other: &CallId) -> bool {
        self == other.0
    }
}

impl PartialEq<&str> for CallId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<CallId> for &str {
    fn eq(&self, other: &CallId) -> bool {
        *self == other.0
    }
}

impl PartialOrd for CallId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CallId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for CallId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl AsRef<str> for CallId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for CallId {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        crate::header::parser::callid(value)
            .map(|(_, call_id)| call_id)
            .map_err(|_| Error::InvalidCallId(value.to_string()))
    }
}
