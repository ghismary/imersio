use std::{cmp::Ordering, hash::Hash};

use crate::{utils::partial_eq_refs, GenericParameter};

#[derive(Clone, Debug, Eq)]
pub enum AcceptParameter {
    Q(String),
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

    pub fn q(&self) -> Option<f32> {
        match self {
            Self::Q(value) => value.parse().ok(),
            _ => None,
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Self::Q(_) => "q",
            Self::Other(value) => value.key(),
        }
    }

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

impl PartialEq<AcceptParameter> for AcceptParameter {
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

partial_eq_refs!(AcceptParameter);

impl Hash for AcceptParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        self.value().map(|v| v.to_ascii_lowercase()).hash(state);
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
        Self::Other(GenericParameter::new(value.key(), value.value()))
    }
}
