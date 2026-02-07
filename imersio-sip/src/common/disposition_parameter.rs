use std::cmp::Ordering;
use std::hash::Hash;

use crate::Handling;
use crate::common::generic_parameter::generic_parameter_display;
use crate::{GenericParameter, TokenString};

/// Representation of a parameter of a `DispositionType`.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum DispositionParameter {
    /// The handling parameter describes how the UAS should react if it
    /// receives a message body whose content type or disposition type it
    /// does not understand.
    Handling(Handling),
    /// Any other parameter.
    Other(GenericParameter<TokenString>),
}

impl DispositionParameter {
    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Handling(_) => "handling",
            Self::Other(param) => param.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Handling(value) => Some(value.value()),
            Self::Other(param) => param.value(),
        }
    }

    /// Get the handling value of the parameter if this is a `handling`
    /// parameter.
    pub fn handling(&self) -> Option<&Handling> {
        match self {
            Self::Handling(value) => Some(value),
            _ => None,
        }
    }
}

impl std::fmt::Display for DispositionParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        generic_parameter_display(self.key(), self.value(), f)
    }
}

impl PartialEq for DispositionParameter {
    fn eq(&self, other: &DispositionParameter) -> bool {
        match (self, other) {
            (Self::Handling(self_handling), Self::Handling(other_handling)) => {
                self_handling == other_handling
            }
            (Self::Other(self_param), Self::Other(other_param)) => self_param == other_param,
            _ => false,
        }
    }
}

impl PartialOrd for DispositionParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DispositionParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
    }
}

impl Hash for DispositionParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state);
        self.value().hash(state);
    }
}

impl From<GenericParameter<TokenString>> for DispositionParameter {
    fn from(value: GenericParameter<TokenString>) -> Self {
        Self::Other(value)
    }
}

pub(crate) mod parser {
    use nom::{
        Parser, branch::alt, bytes::complete::tag_no_case, combinator::map,
        sequence::separated_pair,
    };

    use crate::{
        DispositionParameter, Handling, TokenString,
        common::generic_parameter::parser::generic_param,
        parser::{ParserResult, equal, token},
    };

    #[inline]
    fn other_handling(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }

    fn handling_param(input: &str) -> ParserResult<&str, DispositionParameter> {
        map(
            separated_pair(
                tag_no_case("handling"),
                equal,
                map(
                    alt((
                        map(tag_no_case("optional"), TokenString::new),
                        map(tag_no_case("required"), TokenString::new),
                        other_handling,
                    )),
                    Handling::new,
                ),
            ),
            |(_, value)| DispositionParameter::Handling(value),
        )
        .parse(input)
    }

    pub(crate) fn disp_param(input: &str) -> ParserResult<&str, DispositionParameter> {
        alt((handling_param, map(generic_param, Into::into))).parse(input)
    }
}
