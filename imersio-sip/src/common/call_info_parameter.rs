use std::cmp::Ordering;
use std::hash::Hash;

use crate::common::wrapped_string::WrappedString;
use crate::{GenericParameter, TokenString};

/// Representation of an information about the caller or the callee.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum CallInfoParameter {
    /// The `icon` purpose parameter designates an image suitable as an iconic
    /// representation of the caller or callee.
    IconPurpose,
    /// The `info` purpose parameter describes the caller or callee in general,
    /// for example, through a web page.
    InfoPurpose,
    /// The `card` purpose parameter provides a business card, for example, in
    /// vCard or LDIF formats.
    CardPurpose,
    /// Any other purpose parameter.
    OtherPurpose(TokenString),
    /// Any extension parameter.
    Other(GenericParameter<TokenString>),
}

impl CallInfoParameter {
    pub(crate) fn new(key: TokenString, value: Option<TokenString>) -> Self {
        match (
            key.to_ascii_lowercase().as_str(),
            value.map(|v| v.to_ascii_lowercase()).as_deref(),
        ) {
            ("purpose", Some("icon")) => Self::IconPurpose,
            ("purpose", Some("info")) => Self::InfoPurpose,
            ("purpose", Some("card")) => Self::CardPurpose,
            ("purpose", Some(value)) => Self::OtherPurpose(TokenString::new(value)),
            (key, value) => Self::Other(GenericParameter::new(
                TokenString::new(key),
                value.map(|v| WrappedString::NotWrapped(TokenString::new(v))),
            )),
        }
    }

    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::IconPurpose | Self::InfoPurpose | Self::CardPurpose | Self::OtherPurpose(_) => {
                "purpose"
            }
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::IconPurpose => Some("icon"),
            Self::InfoPurpose => Some("info"),
            Self::CardPurpose => Some("card"),
            Self::OtherPurpose(value) => Some(value),
            Self::Other(value) => value.value(),
        }
    }
}

impl std::fmt::Display for CallInfoParameter {
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

impl PartialEq for CallInfoParameter {
    fn eq(&self, other: &CallInfoParameter) -> bool {
        match (self, other) {
            (Self::IconPurpose, Self::IconPurpose)
            | (Self::InfoPurpose, Self::InfoPurpose)
            | (Self::CardPurpose, Self::CardPurpose) => true,
            (Self::OtherPurpose(a), Self::OtherPurpose(b)) => a.eq_ignore_ascii_case(b),
            (Self::Other(a), Self::Other(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for CallInfoParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CallInfoParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
    }
}

impl Hash for CallInfoParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state);
        self.value().hash(state);
    }
}

impl From<GenericParameter<TokenString>> for CallInfoParameter {
    fn from(value: GenericParameter<TokenString>) -> Self {
        CallInfoParameter::new(
            TokenString::new(value.key()),
            value.value().map(TokenString::new),
        )
    }
}

pub(crate) mod parser {
    use nom::{
        Parser, branch::alt, bytes::complete::tag_no_case, combinator::map, error::context,
        sequence::separated_pair,
    };

    use crate::{
        CallInfoParameter, GenericParameter, TokenString,
        common::{generic_parameter::parser::generic_param, wrapped_string::WrappedString},
        parser::{ParserResult, equal, token},
    };

    pub(crate) fn info_param(input: &str) -> ParserResult<&str, CallInfoParameter> {
        context(
            "info_param",
            map(
                alt((
                    map(
                        separated_pair(
                            map(tag_no_case("purpose"), TokenString::new),
                            equal,
                            map(
                                alt((
                                    map(tag_no_case("icon"), TokenString::new),
                                    map(tag_no_case("info"), TokenString::new),
                                    map(tag_no_case("card"), TokenString::new),
                                    token,
                                )),
                                Some,
                            ),
                        ),
                        |(key, value)| {
                            GenericParameter::new(key, value.map(WrappedString::new_not_wrapped))
                        },
                    ),
                    generic_param,
                )),
                Into::into,
            ),
        )
        .parse(input)
    }
}
