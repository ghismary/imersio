use derive_more::IsVariant;
use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;

use crate::GenericParameter;

/// Representation of a contact parameter.
#[derive(Clone, Debug, Eq, Hash, IsVariant, PartialEq, PartialEqRefs)]
pub enum ContactParameter {
    /// A `q` parameter.
    Q(String),
    /// An `expires` parameter.
    Expires(String),
    /// Any other parameter.
    Other(GenericParameter),
}

impl ContactParameter {
    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Q(_) => "q",
            Self::Expires(_) => "expires",
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Q(value) => Some(value),
            Self::Expires(value) => Some(value),
            Self::Other(value) => value.value(),
        }
    }

    /// Get the q value of the parameter if this is a `q` parameter.
    pub fn q(&self) -> Option<f32> {
        match self {
            Self::Q(value) => value.parse().ok(),
            _ => None,
        }
    }

    /// Get the expires value of the parameter if this is an `expires`
    /// parameter.
    pub fn expires(&self) -> Option<u32> {
        match self {
            Self::Expires(value) => value.parse().ok(),
            _ => None,
        }
    }
}

impl std::fmt::Display for ContactParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.key(),
            if self.value().is_some() { "=" } else { "" },
            self.value().unwrap_or_default()
        )
    }
}

impl PartialOrd for ContactParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContactParameter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
    }
}

impl From<GenericParameter> for ContactParameter {
    fn from(value: GenericParameter) -> Self {
        Self::Other(value)
    }
}
