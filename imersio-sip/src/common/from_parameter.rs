#![allow(missing_docs)]

use derive_more::{Deref, From, IntoIterator, IsVariant};
use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::utils::compare_vectors;
use crate::GenericParameter;

/// Representation of the list of from parameters of a `From` header.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Deref, Eq, From, IntoIterator, PartialEqRefs)]
pub struct FromParameters(Vec<FromParameter>);

impl std::fmt::Display for FromParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", join(&self.0, ";"))
    }
}

impl PartialEq for FromParameters {
    fn eq(&self, other: &Self) -> bool {
        compare_vectors(self.0.deref(), other.0.deref())
    }
}

/// Representation of a parameter founded in a `From` header.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
pub enum FromParameter {
    /// A `tag` parameter.
    Tag(String),
    /// Any other parameters.
    Other(GenericParameter),
}

impl FromParameter {
    /// Get the `tag` value if the parameter is a `tag` parameter.
    pub fn tag(&self) -> Option<&str> {
        match self {
            Self::Tag(value) => Some(value),
            _ => None,
        }
    }

    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Tag(_) => "tag",
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Tag(value) => Some(value),
            Self::Other(value) => value.value(),
        }
    }
}

impl std::fmt::Display for FromParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tag(value) => write!(f, "tag={value}"),
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

impl PartialEq for FromParameter {
    fn eq(&self, other: &FromParameter) -> bool {
        match (self, other) {
            (Self::Tag(a), Self::Tag(b)) => a == b,
            (Self::Other(a), Self::Other(b)) => {
                a.key().eq_ignore_ascii_case(b.key())
                    && a.value().map(|v| v.to_ascii_lowercase())
                        == b.value().map(|v| v.to_ascii_lowercase())
            }
            _ => false,
        }
    }
}

impl PartialOrd for FromParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FromParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .key()
            .to_ascii_lowercase()
            .cmp(&other.key().to_ascii_lowercase())
        {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self {
            Self::Tag(value) => Some(value.as_str()).cmp(&other.value()),
            Self::Other(param) => param
                .value()
                .map(|value| value.to_ascii_lowercase())
                .cmp(&other.value().map(|value| value.to_ascii_lowercase())),
        }
    }
}

impl Hash for FromParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Tag(value) => {
                "tag".hash(state);
                value.hash(state);
            }
            Self::Other(param) => param.hash(state),
        }
    }
}

impl From<GenericParameter> for FromParameter {
    fn from(value: GenericParameter) -> Self {
        match value.key().to_ascii_lowercase().as_str() {
            "tag" => Self::Tag(value.value().unwrap_or("").to_string()),
            _ => Self::Other(value),
        }
    }
}
