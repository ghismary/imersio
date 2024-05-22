//! TODO

use std::cmp::Ordering;

/// Representation of a generic parameter.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenericParameter {
    key: String,
    value: Option<String>,
}

impl GenericParameter {
    /// Create a `GenericParam`.
    pub fn new(key: String, value: Option<String>) -> Self {
        Self { key, value }
    }

    /// Get the key of the `GenericParam`.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the value of the `GenericParam`.
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

impl std::fmt::Display for GenericParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.key,
            if self.value.is_some() { "=" } else { "" },
            self.value.as_deref().unwrap_or_default()
        )
    }
}

impl PartialEq<&GenericParameter> for GenericParameter {
    fn eq(&self, other: &&GenericParameter) -> bool {
        self == *other
    }
}

impl PartialEq<GenericParameter> for &GenericParameter {
    fn eq(&self, other: &GenericParameter) -> bool {
        *self == other
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AcceptParameter {
    Q(String),
    Other(String, Option<String>),
}

impl AcceptParameter {
    pub(crate) fn new(key: String, value: Option<String>) -> Self {
        match (key.as_str(), value.as_deref()) {
            ("q", Some(value)) => Self::Q(value.to_string()),
            _ => Self::Other(key.to_string(), value.map(Into::into)),
        }
    }

    pub fn q(&self) -> Option<f32> {
        match self {
            Self::Q(value) => value.parse().ok(),
            _ => None,
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Self::Q(_) => "q",
            Self::Other(key, _) => key,
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Q(value) => Some(value),
            Self::Other(_, value) => value.as_deref(),
        }
    }
}

impl std::fmt::Display for AcceptParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Q(value) => write!(f, "q={value}"),
            Self::Other(key, value) => write!(
                f,
                "{}{}{}",
                key,
                if value.is_some() { "=" } else { "" },
                value.as_deref().unwrap_or_default()
            ),
        }
    }
}

impl PartialEq<&AcceptParameter> for AcceptParameter {
    fn eq(&self, other: &&AcceptParameter) -> bool {
        self == *other
    }
}

impl PartialEq<AcceptParameter> for &AcceptParameter {
    fn eq(&self, other: &AcceptParameter) -> bool {
        *self == other
    }
}

impl PartialOrd for AcceptParameter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AcceptParameter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
    }
}

impl From<GenericParameter> for AcceptParameter {
    fn from(value: GenericParameter) -> Self {
        Self::Other(value.key().to_string(), value.value().map(Into::into))
    }
}
