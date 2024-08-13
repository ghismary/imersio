#![allow(missing_docs)]

use derive_more::IsVariant;
use nom::error::convert_error;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::uris::parser::hostport;
use crate::Error;
use partial_eq_refs::PartialEqRefs;

/// Representation of a warning agent contained in a Warning header.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
pub enum WarnAgent {
    /// Host + port warning agent.
    HostPort(String),
    /// Pseudonym warning agent.
    Pseudonym(String),
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
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::warn_agent(value) {
            Ok((rest, tag)) => {
                if !rest.is_empty() {
                    Err(Error::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(tag)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(Error::InvalidWarnAgent(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidWarnAgent(format!(
                "Incomplete warning agent `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use crate::parser::{token, ParserResult};
    use crate::uris::parser::hostport;
    use crate::WarnAgent;
    use nom::combinator::consumed;
    use nom::{branch::alt, combinator::map, error::context};

    pub(crate) fn warn_agent(input: &str) -> ParserResult<&str, WarnAgent> {
        context(
            "warn_agent",
            alt((
                map(consumed(hostport), |(hostport, (_, _))| {
                    WarnAgent::HostPort(hostport.to_string())
                }),
                map(pseudonym, |pseudo| WarnAgent::Pseudonym(pseudo.to_string())),
            )),
        )(input)
    }

    #[inline]
    fn pseudonym(input: &str) -> ParserResult<&str, &str> {
        token(input)
    }
}
