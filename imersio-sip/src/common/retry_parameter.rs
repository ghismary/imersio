use derive_more::IsVariant;
use partial_eq_refs::PartialEqRefs;
use std::{cmp::Ordering, hash::Hash};

use crate::GenericParameter;

/// Representation of a parameter contained in a `Retry-After` header.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
pub enum RetryParameter {
    /// duration parameter
    Duration(String),
    /// Any other parameter
    Other(GenericParameter),
}

impl RetryParameter {
    pub(crate) fn new<S: Into<String>>(key: S, value: Option<S>) -> Self {
        let key: String = key.into();
        let value: Option<String> = value.map(Into::into);
        match (key.to_lowercase().as_str(), &value) {
            ("duration", Some(value)) => Self::Duration(value.to_string()),
            _ => Self::Other(GenericParameter::new(key, value)),
        }
    }

    /// Get the value of the duration parameter if it is one.
    pub fn duration(&self) -> Option<u32> {
        match self {
            Self::Duration(value) => value.parse().ok(),
            _ => None,
        }
    }

    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Duration(_) => "duration",
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Duration(value) => Some(value),
            Self::Other(value) => value.value(),
        }
    }
}

impl std::fmt::Display for RetryParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duration(value) => write!(f, "duration={value}"),
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

impl PartialEq for RetryParameter {
    fn eq(&self, other: &RetryParameter) -> bool {
        match (self, other) {
            (Self::Duration(a), Self::Duration(b)) => a == b,
            (Self::Other(a), Self::Other(b)) => {
                a.key().eq_ignore_ascii_case(b.key())
                    && a.value().map(|v| v.to_ascii_lowercase())
                        == b.value().map(|v| v.to_ascii_lowercase())
            }
            _ => false,
        }
    }
}

impl PartialOrd for RetryParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RetryParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl Hash for RetryParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        self.value().map(|v| v.to_ascii_lowercase()).hash(state);
    }
}

impl From<GenericParameter> for RetryParameter {
    fn from(value: GenericParameter) -> Self {
        Self::Other(GenericParameter::new(value.key(), value.value()))
    }
}

pub(crate) mod parser {
    use crate::common::contact_parameter::parser::delta_seconds;
    use crate::common::generic_parameter::parser::generic_param;
    use crate::common::retry_parameter::RetryParameter;
    use crate::parser::{equal, ParserResult};
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{map, recognize},
        error::context,
        sequence::separated_pair,
    };

    pub(crate) fn retry_param(input: &str) -> ParserResult<&str, RetryParameter> {
        context(
            "retry_param",
            alt((
                map(
                    separated_pair(tag_no_case("duration"), equal, recognize(delta_seconds)),
                    |(name, value)| RetryParameter::new(name, Some(value)),
                ),
                map(generic_param, Into::into),
            )),
        )(input)
    }
}
