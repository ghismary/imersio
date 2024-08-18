//! Parsing and generation of the headers of a SIP URI.

use derive_more::{Deref, DerefMut, Display};
use itertools::join;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use crate::parser::ESCAPED_CHARS;
use crate::uris::uri_header::parser::is_hnv_unreserved;
use crate::{parser::is_unreserved, utils::escape, SipError};

/// Representation of a string with limited characters for URI header names.
#[derive(Clone, Debug, Deref, Display, Eq, Hash, PartialEq)]
pub struct UriHeaderNameString(String);

impl UriHeaderNameString {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl TryFrom<&str> for UriHeaderNameString {
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
            Err(SipError::InvalidUriHeaderString(value.to_string()))
        }
    }
}

/// Representation of a string with limited characters for URI header values.
#[derive(Clone, Debug, Deref, Display, Eq, Hash, PartialEq)]
pub struct UriHeaderValueString(String);

impl UriHeaderValueString {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl TryFrom<&str> for UriHeaderValueString {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Do not use the parser because of the escaped characters, instead check that each
        // character of the given value can be escaped.
        if value.chars().all(|c| {
            let idx: Result<u8, _> = c.try_into();
            match idx {
                Ok(idx) => ESCAPED_CHARS[idx as usize] != '\0',
                Err(_) => false,
            }
        }) {
            Ok(Self::new(value))
        } else {
            Err(SipError::InvalidUriHeaderString(value.to_string()))
        }
    }
}

/// Representation of a URI header list.
#[derive(Clone, Debug, Eq)]
pub struct UriHeader {
    name: UriHeaderNameString,
    value: UriHeaderValueString,
}

impl UriHeader {
    pub(crate) fn new(name: UriHeaderNameString, value: UriHeaderValueString) -> Self {
        Self { name, value }
    }

    /// Get the name of the header as a string slice.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the value of the header as a string slice.
    #[inline]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for UriHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}={}",
            escape(self.name(), |c| {
                is_unreserved(c) || is_hnv_unreserved(c)
            }),
            escape(self.value(), |c| {
                is_unreserved(c) || is_hnv_unreserved(c)
            })
        )
    }
}

impl PartialEq for UriHeader {
    fn eq(&self, other: &Self) -> bool {
        self.name().eq_ignore_ascii_case(other.name())
            && self.value().to_ascii_lowercase() == other.value().to_ascii_lowercase()
    }
}

impl PartialOrd for UriHeader {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UriHeader {
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
            .to_ascii_lowercase()
            .cmp(&other.value().to_ascii_lowercase())
    }
}

impl Hash for UriHeader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().to_ascii_lowercase().hash(state);
        self.value().to_ascii_lowercase().hash(state);
    }
}

/// Representation of a list of URI headers.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Default, Deref, DerefMut, Eq)]
pub struct UriHeaders(Vec<UriHeader>);

impl crate::UriHeaders {
    /// Get a URI header by its name.
    pub fn get(&self, name: &str) -> Option<&UriHeader> {
        self.iter().find(|p| p.name().eq_ignore_ascii_case(name))
    }
}

impl std::fmt::Display for crate::UriHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", join(self.deref(), "&"))
    }
}

impl PartialEq for crate::UriHeaders {
    fn eq(&self, other: &Self) -> bool {
        for self_header in &self.0 {
            if let Some(other_header) = other.get(self_header.name()) {
                if !self_header
                    .value()
                    .eq_ignore_ascii_case(other_header.value())
                {
                    return false;
                }
            } else {
                return false;
            }
        }

        for other_header in &other.0 {
            if let Some(self_header) = self.get(other_header.name()) {
                if !other_header
                    .value()
                    .eq_ignore_ascii_case(self_header.value())
                {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl Hash for crate::UriHeaders {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut sorted_headers: Vec<&UriHeader> = self.iter().collect();
        sorted_headers.sort();
        sorted_headers.hash(state);
    }
}

impl From<Vec<UriHeader>> for UriHeaders {
    fn from(value: Vec<UriHeader>) -> Self {
        UriHeaders(value)
    }
}

pub(crate) mod parser {
    use crate::parser::{escaped, take1, unreserved, ParserResult};
    use crate::{UriHeader, UriHeaderNameString, UriHeaderValueString, UriHeaders};
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

    #[inline]
    fn hnv_unreserved(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| is_hnv_unreserved(*c))(input)
    }

    #[inline]
    fn hname(input: &str) -> ParserResult<&str, UriHeaderNameString> {
        context(
            "hname",
            map(many1(alt((hnv_unreserved, unreserved, escaped))), |name| {
                UriHeaderNameString::new(name.iter().collect::<String>())
            }),
        )(input)
    }

    #[inline]
    fn hvalue(input: &str) -> ParserResult<&str, UriHeaderValueString> {
        context(
            "hvalue",
            map(many0(alt((hnv_unreserved, unreserved, escaped))), |value| {
                UriHeaderValueString::new(value.iter().collect::<String>())
            }),
        )(input)
    }

    fn header(input: &str) -> ParserResult<&str, UriHeader> {
        context(
            "header",
            map(separated_pair(hname, tag("="), hvalue), |(name, value)| {
                UriHeader::new(name, value)
            }),
        )(input)
    }

    pub(crate) fn headers(input: &str) -> ParserResult<&str, UriHeaders> {
        context(
            "headers",
            map(
                preceded(tag("?"), separated_list1(tag("&"), header)),
                Into::into,
            ),
        )(input)
    }
}
