//! Parsing and generation of the parameters of a SIP URI.

use std::{hash::Hash, ops::Deref};

use crate::uris::uri_parameters::parser::is_param_unreserved;
use crate::{parser::is_unreserved, utils::escape};

/// Representation of an URI parameters list.
#[derive(Clone, Debug, Default, Eq)]
pub struct UriParameters(Vec<(String, Option<String>)>);

impl UriParameters {
    pub(crate) fn new<S: Into<String>>(parameters: Vec<(S, Option<S>)>) -> Self {
        Self(
            parameters
                .into_iter()
                .map(|(key, value)| (key.into(), value.map(Into::into)))
                .collect::<Vec<(String, Option<String>)>>(),
        )
    }

    /// Tell whether the parameters list is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of parameters.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Tell whether the parameters list contains a parameter with the given
    /// name.
    pub fn contains(&self, name: &str) -> bool {
        self.iter().any(|(n, _)| n == name)
    }

    /// Get the parameter corresponding to the given name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .and_then(|(_, v)| v.as_deref())
    }
}

impl std::fmt::Display for UriParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}{}{}",
                        escape(k, |b| { is_unreserved(b) || is_param_unreserved(b) }),
                        if v.is_some() { "=" } else { "" },
                        escape(v.as_deref().unwrap_or_default(), |b| {
                            is_unreserved(b) || is_param_unreserved(b)
                        })
                    )
                })
                .collect::<Vec<String>>()
                .join(";"),
        )
    }
}

impl Deref for UriParameters {
    type Target = Vec<(String, Option<String>)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for UriParameters {
    fn eq(&self, other: &Self) -> bool {
        for (sk, sv) in &self.0 {
            for (ok, ov) in &other.0 {
                if sk.eq_ignore_ascii_case(ok)
                    && sv.as_ref().map(|s| s.to_ascii_lowercase())
                        != ov.as_ref().map(|s| s.to_ascii_lowercase())
                {
                    return false;
                }
            }
        }

        let stransport = self.get("transport");
        let otransport = other.get("transport");
        match (stransport, otransport) {
            (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}

impl Hash for UriParameters {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut sorted_params: Vec<(String, Option<String>)> = self
            .0
            .iter()
            .map(|(key, value)| {
                (
                    key.to_ascii_lowercase(),
                    value.as_ref().map(|value| value.to_ascii_lowercase()),
                )
            })
            .collect();
        sorted_params.sort_by(|(a, _), (b, _)| a.cmp(b));
        sorted_params.hash(state)
    }
}

pub(crate) mod parser {
    use crate::parser::{escaped, take1, token, ttl, unreserved, ParserResult};
    use crate::uris::host::parser::host;
    use crate::UriParameters;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, opt, verify},
        error::context,
        multi::{many0, many1},
        sequence::{pair, preceded, separated_pair},
    };

    fn transport_param(input: &str) -> ParserResult<&str, (String, String)> {
        context(
            "transport_param",
            map(
                separated_pair(tag("transport"), tag("="), token),
                |(name, value)| (name.to_string(), value.to_string()),
            ),
        )(input)
    }

    fn user_param(input: &str) -> ParserResult<&str, (String, String)> {
        context(
            "user_param",
            map(
                separated_pair(tag("user"), tag("="), token),
                |(name, value)| (name.to_string(), value.to_string()),
            ),
        )(input)
    }

    fn method_param(input: &str) -> ParserResult<&str, (String, String)> {
        context(
            "method_param",
            map(
                separated_pair(tag("method"), tag("="), token),
                |(name, value)| (name.to_string(), value.to_string()),
            ),
        )(input)
    }

    fn ttl_param(input: &str) -> ParserResult<&str, (String, String)> {
        context(
            "ttl_param",
            map(
                separated_pair(tag("ttl"), tag("="), ttl),
                |(name, value)| (name.to_string(), value.to_string()),
            ),
        )(input)
    }

    fn maddr_param(input: &str) -> ParserResult<&str, (String, String)> {
        context(
            "maddr_param",
            map(
                separated_pair(tag("maddr"), tag("="), host),
                |(name, value)| (name.to_string(), value.to_string()),
            ),
        )(input)
    }

    #[inline]
    fn lr_param(input: &str) -> ParserResult<&str, (String, String)> {
        context(
            "lr_param",
            map(tag("lr"), |name: &str| (name.to_string(), "".to_string())),
        )(input)
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

    fn other_param(input: &str) -> ParserResult<&str, (String, String)> {
        context(
            "other_param",
            map(
                pair(pname, opt(preceded(tag("="), pvalue))),
                |(name, value)| (name, value.unwrap_or_default()),
            ),
        )(input)
    }

    fn uri_parameter(input: &str) -> ParserResult<&str, (String, String)> {
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
            map(many0(preceded(tag(";"), uri_parameter)), |parameters| {
                UriParameters::new(
                    parameters
                        .into_iter()
                        .map(|(k, v)| (k, if v.is_empty() { None } else { Some(v) }))
                        .collect(),
                )
            }),
        )(input)
    }
}
