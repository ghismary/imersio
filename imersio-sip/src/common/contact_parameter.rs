use std::cmp::Ordering;

use crate::{GenericParameter, TokenString};

/// Representation of a contact parameter.
#[derive(Clone, Debug, Eq, Hash, PartialEq, derive_more::IsVariant)]
pub enum ContactParameter {
    /// A `q` parameter.
    Q(String),
    /// An `expires` parameter.
    Expires(String),
    /// Any other parameter.
    Other(GenericParameter<TokenString>),
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
    fn cmp(&self, other: &Self) -> Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
    }
}

impl From<GenericParameter<TokenString>> for ContactParameter {
    fn from(value: GenericParameter<TokenString>) -> Self {
        Self::Other(value)
    }
}

pub(crate) mod parser {
    use chrono::TimeDelta;
    use nom::{
        Parser,
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{map, recognize},
        multi::many1,
        sequence::separated_pair,
    };

    use crate::{
        ContactParameter, GenericParameter, TokenString,
        common::{accept_parameter::parser::qvalue, generic_parameter::parser::generic_param},
        parser::{ParserResult, digit, equal},
    };

    fn c_p_q(input: &str) -> ParserResult<&str, ContactParameter> {
        map(
            separated_pair(tag_no_case("q"), equal, qvalue),
            |(_, value)| ContactParameter::Q(value.to_string()),
        )
        .parse(input)
    }

    #[inline]
    pub(crate) fn delta_seconds(input: &str) -> ParserResult<&str, TimeDelta> {
        map(recognize(many1(digit)), |digits| {
            TimeDelta::seconds(digits.parse::<u32>().unwrap_or(u32::MAX) as i64)
        })
        .parse(input)
    }

    fn c_p_expires(input: &str) -> ParserResult<&str, ContactParameter> {
        map(
            separated_pair(
                tag_no_case("expires"),
                equal,
                map(delta_seconds, |seconds| seconds.num_seconds().to_string()),
            ),
            |(_, value)| ContactParameter::Expires(value),
        )
        .parse(input)
    }

    #[inline]
    fn contact_extension(input: &str) -> ParserResult<&str, GenericParameter<TokenString>> {
        generic_param(input)
    }

    pub(crate) fn contact_params(input: &str) -> ParserResult<&str, ContactParameter> {
        alt((c_p_q, c_p_expires, map(contact_extension, Into::into))).parse(input)
    }
}
