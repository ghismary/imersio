use std::{cmp::Ordering, hash::Hash};

use crate::common::wrapped_string::WrappedString;
use crate::{GenericParameter, TokenString};

/// Representation of a parameter for a contact contained in an `Accept` header.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum AcceptParameter {
    /// q parameter
    Q(TokenString),
    /// Any other parameter
    Other(GenericParameter<TokenString>),
}

impl AcceptParameter {
    pub(crate) fn new(key: TokenString, value: Option<TokenString>) -> Self {
        match (key.to_lowercase().as_str(), &value) {
            ("q", Some(value)) => Self::Q(value.clone()),
            _ => Self::Other(GenericParameter::new(
                key,
                value.map(WrappedString::new_not_wrapped),
            )),
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

impl From<GenericParameter<TokenString>> for AcceptParameter {
    fn from(value: GenericParameter<TokenString>) -> Self {
        Self::Other(value)
    }
}

pub(crate) mod parser {
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, opt, recognize},
        error::context,
        multi::many_m_n,
        sequence::{pair, separated_pair},
        Parser,
    };

    use crate::{
        common::generic_parameter::parser::generic_param,
        parser::{digit, equal, ParserResult},
        AcceptParameter, TokenString,
    };

    pub(crate) fn qvalue(input: &str) -> ParserResult<&str, TokenString> {
        context(
            "qvalue",
            map(
                recognize(alt((
                    pair(
                        tag("0"),
                        opt(pair(tag("."), many_m_n(0, 3, recognize(digit)))),
                    ),
                    pair(tag("1"), opt(pair(tag("."), many_m_n(0, 3, tag("0"))))),
                ))),
                TokenString::new,
            ),
        )
        .parse(input)
    }

    fn q_param(input: &str) -> ParserResult<&str, AcceptParameter> {
        context(
            "q_param",
            map(
                separated_pair(map(tag("q"), TokenString::new), equal, qvalue),
                |(key, value)| AcceptParameter::new(key, Some(value)),
            ),
        )
        .parse(input)
    }

    pub(crate) fn accept_param(input: &str) -> ParserResult<&str, AcceptParameter> {
        context(
            "accept_param",
            alt((q_param, map(generic_param, Into::into))),
        )
        .parse(input)
    }
}
