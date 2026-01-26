use std::cmp::Ordering;
use std::hash::Hash;

use crate::TokenString;

/// Representation of the `handling` parameter of a `DispositionType`.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum Handling {
    /// The handling of the content type is optional.
    Optional,
    /// The handling of the content type is required.
    Required,
    /// Any extension value.
    Other(TokenString),
}

impl Handling {
    pub(crate) fn new(handling: TokenString) -> Handling {
        match handling.to_ascii_lowercase().as_str() {
            "optional" => Self::Optional,
            "required" => Self::Required,
            _ => Self::Other(handling),
        }
    }

    /// Get the value of the `HandlingValue.`
    pub fn value(&self) -> &str {
        match self {
            Self::Optional => "optional",
            Self::Required => "required",
            Self::Other(value) => value,
        }
    }
}

impl std::fmt::Display for Handling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq for Handling {
    fn eq(&self, other: &Handling) -> bool {
        match (self, other) {
            (Self::Optional, Self::Optional) | (Self::Required, Self::Required) => true,
            (Self::Other(self_value), Self::Other(other_value)) => {
                self_value.eq_ignore_ascii_case(other_value)
            }
            _ => false,
        }
    }
}

impl PartialOrd for Handling {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Handling {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(other.value())
    }
}

impl Hash for Handling {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}
