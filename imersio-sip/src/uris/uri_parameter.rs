//! Parsing and generation of the parameters of a SIP URI.

use itertools::{Itertools, join};
use nom_language::error::convert_error;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::parser::ESCAPED_CHARS;
use crate::uris::uri_parameter::parser::{is_param_unreserved, uri_parameter};
use crate::{
    GenericParameter, Host, Method, SipError, Transport, UserType, parser::is_unreserved,
    utils::escape,
};

/// Representation of a URI user value accepting only the valid characters.
#[derive(Clone, Debug, Eq, Hash, PartialEq, derive_more::Deref, derive_more::Display)]
pub struct UriParameterString(String);

impl UriParameterString {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for UriParameterString {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<&str> for UriParameterString {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Do not use the parser because of the escaped characters, instead check that each
        // character of the given value can be escaped.
        if !value.is_empty()
            && value.chars().all(|c| {
                let idx: Result<u8, _> = c.try_into();
                match idx {
                    Ok(idx) => ESCAPED_CHARS[idx as usize] != '\0',
                    Err(_) => false,
                }
            })
        {
            Ok(Self::new(value))
        } else {
            Err(SipError::InvalidUriParameter(value.to_string()))
        }
    }
}

/// Representation of a SIP URI parameter.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum UriParameter {
    /// A `transport` parameter.
    Transport(Transport),
    /// A `user` parameter.
    User(UserType),
    /// A `method` parameter.
    Method(Method),
    /// A `ttl` parameter.
    Ttl(u8),
    /// A `maddr` parameter.
    MAddr(Host),
    /// A `lr` parameter.
    Lr,
    /// Any other parameter.
    Other(GenericParameter<UriParameterString>),
}

impl UriParameter {
    /// Get the name of the parameter as a string slice.
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

    /// Get the value of the parameter as a string.
    pub fn value(&self) -> Option<String> {
        match self {
            Self::Transport(value) => Some(value.value().to_string().to_ascii_lowercase()),
            Self::User(value) => Some(value.value().to_string()),
            Self::Method(value) => Some(value.to_string()),
            Self::Ttl(value) => Some(format!("{value}")),
            Self::MAddr(value) => Some(value.to_string()),
            Self::Lr => None,
            Self::Other(value) => value.value().map(Into::into),
        }
    }

    /// Get the value of the `transport` parameter if this is one.
    pub fn transport(&self) -> Option<&Transport> {
        match self {
            Self::Transport(value) => Some(value),
            _ => None,
        }
    }

    /// Get the value of the `user` parameter if this is one.
    pub fn user(&self) -> Option<&UserType> {
        match self {
            Self::User(value) => Some(value),
            _ => None,
        }
    }

    /// Get the value of the `method` parameter if this is one.
    pub fn method(&self) -> Option<&Method> {
        match self {
            Self::Method(value) => Some(value),
            _ => None,
        }
    }

    /// Get the value of the `ttl` parameter if this is one.
    pub fn ttl(&self) -> Option<u8> {
        match self {
            Self::Ttl(value) => Some(*value),
            _ => None,
        }
    }

    /// Get the value of the `maddr` parameter if this one.
    pub fn maddr(&self) -> Option<&Host> {
        match self {
            Self::MAddr(value) => Some(value),
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
            escape(self.value().unwrap_or_default().as_str(), |b| {
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

impl From<GenericParameter<UriParameterString>> for UriParameter {
    fn from(value: GenericParameter<UriParameterString>) -> Self {
        Self::Other(value)
    }
}

impl TryFrom<&str> for UriParameter {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match uri_parameter(value) {
            Ok((rest, parameter)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(parameter)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidUriParameter(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidUriParameter(format!(
                "Incomplete uri parameter `{}`",
                value
            ))),
        }
    }
}

/// Representation of a list of URI parameters.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Default, Eq, derive_more::Deref, derive_more::DerefMut)]
pub struct UriParameters(Vec<UriParameter>);

impl UriParameters {
    /// Get a URI parameter by its name.
    pub fn get(&self, name: &str) -> Option<&UriParameter> {
        self.iter().find(|p| p.name().eq_ignore_ascii_case(name))
    }
}

impl UriParameters {
    pub(crate) fn add_parameter(&mut self, parameter: UriParameter) {
        let previous_parameter = self
            .iter()
            .find_position(|p| p.name().eq_ignore_ascii_case(parameter.name()));
        if let Some((idx, _)) = previous_parameter {
            self.remove(idx);
        }
        self.push(parameter);
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
    fn hash<H: Hasher>(&self, state: &mut H) {
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
    use nom::{
        Parser,
        branch::alt,
        bytes::complete::tag,
        combinator::{map, map_res, opt, value, verify},
        error::context,
        multi::{many0, many1},
        sequence::{pair, preceded, separated_pair},
    };

    use crate::{
        GenericParameter, Method, Transport, UriParameter, UriParameterString, UriParameters,
        UserType,
        common::wrapped_string::WrappedString,
        parser::{ParserResult, escaped, take1, token, ttl, unreserved},
        uris::host::parser::host,
    };

    fn transport_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "transport_param",
            map(
                separated_pair(tag("transport"), tag("="), token),
                |(_, value)| UriParameter::Transport(Transport::new(value)),
            ),
        )
        .parse(input)
    }

    fn user_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "user_param",
            map(
                separated_pair(tag("user"), tag("="), token),
                |(_, value)| UriParameter::User(UserType::new(value)),
            ),
        )
        .parse(input)
    }

    fn method_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "method_param",
            map(
                separated_pair(tag("method"), tag("="), token),
                |(_, value)| UriParameter::Method(Method::new(value)),
            ),
        )
        .parse(input)
    }

    fn ttl_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "ttl_param",
            map(separated_pair(tag("ttl"), tag("="), ttl), |(_, value)| {
                UriParameter::Ttl(value)
            }),
        )
        .parse(input)
    }

    fn maddr_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "maddr_param",
            map(
                separated_pair(tag("maddr"), tag("="), host),
                |(_, value)| UriParameter::MAddr(value),
            ),
        )
        .parse(input)
    }

    #[inline]
    fn lr_param(input: &str) -> ParserResult<&str, UriParameter> {
        context("lr_param", value(UriParameter::Lr, tag("lr"))).parse(input)
    }

    #[inline]
    pub(crate) fn is_param_unreserved(c: char) -> bool {
        "[]/:&+$".contains(c)
    }

    fn param_unreserved(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| is_param_unreserved(*c)).parse(input)
    }

    fn paramchar(input: &str) -> ParserResult<&str, char> {
        alt((param_unreserved, unreserved, escaped)).parse(input)
    }

    fn pname(input: &str) -> ParserResult<&str, UriParameterString> {
        context(
            "pname",
            map(many1(paramchar), |pname| {
                UriParameterString::new(pname.iter().collect::<String>())
            }),
        )
        .parse(input)
    }

    fn pvalue(input: &str) -> ParserResult<&str, UriParameterString> {
        context(
            "pvalue",
            map(many1(paramchar), |pvalue| {
                UriParameterString::new(pvalue.iter().collect::<String>())
            }),
        )
        .parse(input)
    }

    fn other_param(input: &str) -> ParserResult<&str, UriParameter> {
        context(
            "other_param",
            map(
                pair(pname, opt(preceded(tag("="), pvalue))),
                |(name, value)| {
                    UriParameter::Other(GenericParameter::new(
                        name,
                        value.map(WrappedString::new_not_wrapped),
                    ))
                },
            ),
        )
        .parse(input)
    }

    pub(super) fn uri_parameter(input: &str) -> ParserResult<&str, UriParameter> {
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
        )
        .parse(input)
    }

    pub(crate) fn uri_parameters(input: &str) -> ParserResult<&str, UriParameters> {
        context(
            "uri_parameters",
            map_res(many0(preceded(tag(";"), uri_parameter)), TryInto::try_into),
        )
        .parse(input)
    }
}
