use std::cmp::Ordering;

use crate::headers::HeaderAccessor;
use crate::TokenString;

#[derive(Clone, Debug, Eq)]
pub struct GenericHeader {
    name: TokenString,
    separator: String,
    value: String,
}

impl GenericHeader {
    pub(crate) fn new<S: Into<String>>(name: TokenString, separator: S, value: S) -> Self {
        Self {
            name,
            separator: separator.into(),
            value: value.into(),
        }
    }
}

impl HeaderAccessor for GenericHeader {
    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn separator(&self) -> &str {
        self.separator.as_str()
    }
    fn value(&self) -> &str {
        self.value.as_str()
    }

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        None
    }
    fn normalized_value(&self) -> String {
        self.value.clone()
    }
}

impl std::fmt::Display for GenericHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.name, self.separator, self.value)
    }
}

impl PartialEq for GenericHeader {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq_ignore_ascii_case(&other.name) && self.value.eq_ignore_ascii_case(&other.value)
    }
}

impl PartialOrd for GenericHeader {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GenericHeader {
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .name
            .to_ascii_lowercase()
            .cmp(&other.name.to_ascii_lowercase())
        {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value
            .to_ascii_lowercase()
            .cmp(&other.value.to_ascii_lowercase())
    }
}

pub(crate) mod parser {
    use nom::{
        branch::alt,
        combinator::{map, recognize},
        error::context,
        multi::many0,
        Parser,
    };

    use crate::{
        headers::GenericHeader,
        parser::{hcolon, lws, text_utf8char, token, ParserResult},
        Header, TokenString,
    };

    #[inline]
    fn header_name(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }

    #[inline]
    fn header_value(input: &str) -> ParserResult<&str, &str> {
        context(
            "header_value",
            recognize(many0(alt((recognize(text_utf8char), lws)))),
        )
        .parse(input)
    }

    pub(crate) fn extension_header(input: &str) -> ParserResult<&str, Header> {
        map(
            (header_name, hcolon, header_value),
            |(name, separator, value)| {
                Header::ExtensionHeader(GenericHeader::new(name, separator, value))
            },
        )
        .parse(input)
    }
}
