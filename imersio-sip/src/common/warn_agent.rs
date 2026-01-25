use nom_language::error::convert_error;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::uris::host::parser::hostport;
use crate::{SipError, TokenString};

/// Representation of a warning agent contained in a Warning header.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum WarnAgent {
    /// Host + port warning agent.
    HostPort(String),
    /// Pseudonym warning agent.
    Pseudonym(TokenString),
}

impl WarnAgent {
    /// Get the value of the warning agent.
    pub fn value(&self) -> &str {
        match self {
            Self::HostPort(value) => value,
            Self::Pseudonym(value) => value,
        }
    }
}

impl std::fmt::Display for WarnAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq<WarnAgent> for WarnAgent {
    fn eq(&self, other: &WarnAgent) -> bool {
        match (self, other) {
            (Self::HostPort(a), Self::HostPort(b)) => hostport(a) == hostport(b),
            (Self::Pseudonym(a), Self::Pseudonym(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for WarnAgent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WarnAgent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(other.value())
    }
}

impl Hash for WarnAgent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}

impl TryFrom<&str> for WarnAgent {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::warn_agent(value) {
            Ok((rest, tag)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(tag)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidWarnAgent(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidWarnAgent(format!(
                "Incomplete warning agent `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use nom::{
        branch::alt,
        combinator::{consumed, map},
        error::context,
        Parser,
    };

    use crate::{
        parser::{token, ParserResult},
        uris::host::parser::hostport,
        TokenString, WarnAgent,
    };

    pub(crate) fn warn_agent(input: &str) -> ParserResult<&str, WarnAgent> {
        context(
            "warn_agent",
            alt((
                map(consumed(hostport), |(hostport, (_, _))| {
                    WarnAgent::HostPort(hostport.to_string())
                }),
                map(pseudonym, WarnAgent::Pseudonym),
            )),
        )
        .parse(input)
    }

    #[inline]
    fn pseudonym(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }
}
