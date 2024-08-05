#![allow(missing_docs)]

use derive_more::IsVariant;
use partial_eq_refs::PartialEqRefs;
use std::{cmp::Ordering, hash::Hash};

use crate::GenericParameter;

/// Representation of a parameter for a contact contained in a `Accept` header.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
pub enum AcceptParameter {
    /// q parameter
    Q(String),
    /// Any other parameter
    Other(GenericParameter),
}

impl AcceptParameter {
    pub(crate) fn new<S: Into<String>>(key: S, value: Option<S>) -> Self {
        let key: String = key.into();
        let value: Option<String> = value.map(Into::into);
        match (key.as_str(), &value) {
            ("q", Some(value)) => Self::Q(value.to_string()),
            _ => Self::Other(GenericParameter::new(key, value)),
        }
    }

    /// Get the value of the q parameter if it is one.
    pub fn q(&self) -> Option<f32> {
        match self {
            Self::Q(value) => value.parse().ok(),
            _ => None,
        }
    }

    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Q(_) => "q",
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Q(value) => Some(value),
            Self::Other(value) => value.value(),
        }
    }
}

impl std::fmt::Display for AcceptParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Q(value) => write!(f, "q={value}"),
            Self::Other(value) => write!(
                f,
                "{}{}{}",
                value.key().to_ascii_lowercase(),
                if value.value().is_some() { "=" } else { "" },
                value.value().unwrap_or_default().to_ascii_lowercase()
            ),
        }
    }
}

impl PartialEq for AcceptParameter {
    fn eq(&self, other: &AcceptParameter) -> bool {
        match (self, other) {
            (Self::Q(a), Self::Q(b)) => a == b,
            (Self::Other(a), Self::Other(b)) => {
                a.key().eq_ignore_ascii_case(b.key())
                    && a.value().map(|v| v.to_ascii_lowercase())
                        == b.value().map(|v| v.to_ascii_lowercase())
            }
            _ => false,
        }
    }
}

impl PartialOrd for AcceptParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AcceptParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl Hash for AcceptParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        self.value().map(|v| v.to_ascii_lowercase()).hash(state);
    }
}

impl From<GenericParameter> for AcceptParameter {
    fn from(value: GenericParameter) -> Self {
        Self::Other(GenericParameter::new(value.key(), value.value()))
    }
}
