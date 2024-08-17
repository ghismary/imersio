//! Parsing and generation of the parameters of a SIP URI.

use derive_more::{Deref, IsVariant};
use itertools::join;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::uris::host::parser::host;
use crate::uris::uri_parameter::parser::is_param_unreserved;
use crate::{
    parser::is_unreserved, utils::escape, GenericParameter, Host, Method, SipError, Transport,
    UserType,
};

/// Representation of a SIP URI parameter.
#[derive(Clone, Debug, Eq, IsVariant)]
pub enum UriParameter {
    /// A `transport` parameter.
    Transport(String),
    /// A `user` parameter.
    User(String),
    /// A `method` parameter.
    Method(String),
    /// A `ttl` parameter.
    Ttl(String),
    /// A `maddr` parameter.
    MAddr(String),
    /// A `lr` parameter.
    Lr,
    /// Any other parameter.
    Other(GenericParameter),
}

impl UriParameter {
    /// Get the name of the parameter.
    pub fn name(&self) -> &str {
        match self {
            Self::Transport(_) => "transport",
            Self::User(_) => "user",
            Self::Method(_) => "method",
            Self::Ttl(_) => "ttl",
            Self::MAddr(_) => "maddr",
            Self::Lr => "lr",
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Transport(value) => Some(value),
            Self::User(value) => Some(value),
            Self::Method(value) => Some(value),
            Self::Ttl(value) => Some(value),
            Self::MAddr(value) => Some(value),
            Self::Lr => None,
            Self::Other(value) => value.value(),
        }
    }

    /// Get the value of the `transport` parameter if this is one.
    pub fn transport(&self) -> Option<Transport> {
        match self {
            Self::Transport(value) => Some(value.as_str().into()),
            _ => None,
        }
    }

    /// Get the value of the `user` parameter if this is one.
    pub fn user(&self) -> Option<UserType> {
        match self {
            Self::User(value) => Some(value.as_str().into()),
            _ => None,
        }
    }

    /// Get the value of the `method` parameter if this is one.
    pub fn method(&self) -> Option<Method> {
        match self {
            Self::Method(value) => Some(value.as_str().try_into().unwrap()),
            _ => None,
        }
    }

    /// Get the value of the `ttl` parameter if this is one.
    pub fn ttl(&self) -> Option<u8> {
        match self {
            Self::Ttl(value) => value.parse().ok(),
            _ => None,
        }
    }

    /// Get the value of the `maddr` parameter if this one.
    pub fn maddr(&self) -> Option<Host> {
        match self {
            Self::MAddr(value) => host(value).ok().map(|(_, host)| host),
            _ => None,
        }
    }
}

impl std::fmt::Display for UriParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            escape(self.name(), |b| {
                is_unreserved(b) || is_param_unreserved(b)
            }),
            if self.value().is_some() { "=" } else { "" },
            escape(self.value().unwrap_or_default(), |b| {
                is_unreserved(b) || is_param_unreserved(b)
            })
        )
    }
}

impl PartialEq for UriParameter {
    fn eq(&self, other: &Self) -> bool {
        self.name().eq_ignore_ascii_case(other.name())
            && self.value().map(|s| s.to_ascii_lowercase())
                == other.value().map(|s| s.to_ascii_lowercase())
    }
}

impl PartialOrd for UriParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UriParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .name()
            .to_ascii_lowercase()
            .cmp(&other.name().to_ascii_lowercase())
        {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value()
            .unwrap()
            .to_ascii_lowercase()
            .cmp(&other.value().unwrap().to_ascii_lowercase())
    }
}

impl Hash for UriParameter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().to_ascii_lowercase().hash(state);
        self.value().map(|v| v.to_ascii_lowercase()).hash(state);
    }
}

impl From<GenericParameter> for UriParameter {
    fn from(value: GenericParameter) -> Self {
        Self::Other(value)
    }
}

/// Representation of a list of URI parameters.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Default, Deref, Eq)]
pub struct UriParameters(Vec<UriParameter>);

impl UriParameters {
    /// Get a URI parameter by its name.
    pub fn get(&self, name: &str) -> Option<&UriParameter> {
        self.iter().find(|p| p.name().eq_ignore_ascii_case(name))
    }
}

impl std::fmt::Display for UriParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", join(self.deref(), ";"))
    }
}

impl PartialEq for UriParameters {
    fn eq(&self, other: &Self) -> bool {
        for self_param in &self.0 {
            for other_param in &other.0 {
                if self_param.name().eq_ignore_ascii_case(other_param.name())
                    && self_param.value().map(|s| s.to_ascii_lowercase())
                        != other_param.value().map(|s| s.to_ascii_lowercase())
                {
                    return false;
                }
            }
        }

        let self_transport = self.get("transport");
        let other_transport = other.get("transport");
        match (self_transport, other_transport) {
            (Some(a), Some(b)) => a == b,
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}

impl Hash for UriParameters {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut sorted_params: Vec<&UriParameter> = self.iter().collect();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

impl TryFrom<Vec<UriParameter>> for UriParameters {
    type Error = SipError;

    fn try_from(value: Vec<UriParameter>) -> Result<Self, Self::Error> {
        let mut uniq = HashSet::new();
        if value
            .iter()
            .all(|p| uniq.insert(p.name().to_ascii_lowercase()))
        {
            Ok(Self(value))
        } else {
            Err(SipError::DuplicatedUriParameters)
        }
    }
}

pub(crate) mod parser {
    use crate::parser::{escaped, take1, token, ttl, unreserved, ParserResult};
    use crate::uris::host::parser::host;
    use crate::{GenericParameter, UriParameter, UriParameters};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, map_res, opt, value, verify},
        error::context,
        multi::{many0, many1},
        sequence::{pair, preceded, separated_pair},
    };

    fn transport_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "transport_param",
            map(
                separated_pair(tag("transport"), tag("="), token),
                |(_, value)| UriParameter::Transport(value.to_string()),
            ),
        )(input)
    }

    fn user_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "user_param",
            map(
                separated_pair(tag("user"), tag("="), token),
                |(_, value)| UriParameter::User(value.to_string()),
            ),
        )(input)
    }

    fn method_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "method_param",
            map(
                separated_pair(tag("method"), tag("="), token),
                |(_, value)| UriParameter::Method(value.to_string()),
            ),
        )(input)
    }

    fn ttl_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "ttl_param",
            map(separated_pair(tag("ttl"), tag("="), ttl), |(_, value)| {
                UriParameter::Ttl(value.to_string())
            }),
        )(input)
    }

    fn maddr_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "maddr_param",
            map(
                separated_pair(tag("maddr"), tag("="), host),
                |(_, value)| UriParameter::MAddr(value.to_string()),
            ),
        )(input)
    }

    #[inline]
    fn lr_param(input: &str) -> ParserResult<&str, UriParameter> {
        context("lr_param", value(UriParameter::Lr, tag("lr")))(input)
    }

    #[inline]
    pub(crate) fn is_param_unreserved(c: char) -> bool {
        "[]/:&+$".contains(c)
    }

    fn param_unreserved(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| is_param_unreserved(*c))(input)
    }

    fn paramchar(input: &str) -> ParserResult<&str, char> {
        alt((param_unreserved, unreserved, escaped))(input)
    }

    fn pname(input: &str) -> ParserResult<&str, String> {
        context(
            "pname",
            map(many1(paramchar), |pname| pname.iter().collect::<String>()),
        )(input)
    }

    fn pvalue(input: &str) -> ParserResult<&str, String> {
        context(
            "pvalue",
            map(many1(paramchar), |pvalue| pvalue.iter().collect::<String>()),
        )(input)
    }

    fn other_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "other_param",
            map(
                pair(pname, opt(preceded(tag("="), pvalue))),
                |(name, value)| UriParameter::Other(GenericParameter::new(name, value)),
            ),
        )(input)
    }

    fn uri_parameter(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "uri_parameter",
            alt((
                transport_param,
                user_param,
                method_param,
                ttl_param,
                maddr_param,
                lr_param,
                other_param,
            )),
        )(input)
    }

    pub(crate) fn uri_parameters(input: &str) -> ParserResult<&str, UriParameters> {
        context(
            "uri_parameters",
            map_res(many0(preceded(tag(";"), uri_parameter)), TryInto::try_into),
        )(input)
    }
}
