//! Parsing and generation of the headers of a SIP URI.

use std::hash::Hash;

use crate::uris::uri_headers::parser::is_hnv_unreserved;
use crate::{parser::is_unreserved, utils::escape};

/// Representation of an URI header list.
#[derive(Clone, Debug, Default, Eq)]
pub struct UriHeaders(Vec<(String, String)>);

impl UriHeaders {
    pub(crate) fn new<S: Into<String>>(headers: Vec<(S, S)>) -> Self {
        Self(
            headers
                .into_iter()
                .map(|(n, v)| (n.into(), v.into()))
                .collect(),
        )
    }

    /// Tell whether the headers list is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of headers.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Tell whether the headers list contains a header with the given name.
    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|(n, _)| n == name)
    }

    /// Get the header corresponding to the given name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_ref())
    }
}

impl std::fmt::Display for UriHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        escape(k, |c| { is_unreserved(c) || is_hnv_unreserved(c) }),
                        escape(v, |c| { is_unreserved(c) || is_hnv_unreserved(c) })
                    )
                })
                .collect::<Vec<String>>()
                .join("&"),
        )
    }
}

impl PartialEq for UriHeaders {
    fn eq(&self, other: &Self) -> bool {
        for (sk, sv) in &self.0 {
            if let Some(ov) = other.get(sk) {
                if sv != ov {
                    return false;
                }
            } else {
                return false;
            }
        }

        for (ok, ov) in &other.0 {
            if let Some(sv) = self.get(ok) {
                if ov != sv {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl Hash for UriHeaders {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut sorted_headers: Vec<(String, String)> = self
            .0
            .iter()
            .map(|(key, value)| (key.to_ascii_lowercase(), value.to_ascii_lowercase()))
            .collect();
        sorted_headers.sort_by(|(a, _), (b, _)| a.cmp(b));
        sorted_headers.hash(state)
    }
}

pub(crate) mod parser {
    use crate::parser::{escaped, take1, unreserved, ParserResult};
    use crate::UriHeaders;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, verify},
        error::context,
        multi::{many0, many1, separated_list1},
        sequence::{preceded, separated_pair},
    };

    #[inline]
    pub(crate) fn is_hnv_unreserved(c: char) -> bool {
        "[]/?:+$".contains(c)
    }

    fn hnv_unreserved(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| is_hnv_unreserved(*c))(input)
    }

    fn hname(input: &str) -> ParserResult<&str, String> {
        context(
            "hname",
            map(many1(alt((hnv_unreserved, unreserved, escaped))), |name| {
                name.iter().collect::<String>()
            }),
        )(input)
    }

    fn hvalue(input: &str) -> ParserResult<&str, String> {
        context(
            "hvalue",
            map(many0(alt((hnv_unreserved, unreserved, escaped))), |value| {
                value.iter().collect::<String>()
            }),
        )(input)
    }

    fn header(input: &str) -> ParserResult<&str, (String, String)> {
        context("header", separated_pair(hname, tag("="), hvalue))(input)
    }

    pub(crate) fn headers(input: &str) -> ParserResult<&str, UriHeaders> {
        context(
            "headers",
            map(
                preceded(tag("?"), separated_list1(tag("&"), header)),
                UriHeaders::new,
            ),
        )(input)
    }
}
