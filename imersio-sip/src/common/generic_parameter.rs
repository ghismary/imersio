use crate::common::wrapped_string::WrappedString;
use crate::utils::compare_vectors;
use derive_more::{Deref, From, IntoIterator};
use itertools::join;
use std::cmp::Ordering;
use std::hash::Hash;

/// Representation of the list of generic parameters.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Deref, Eq, From, IntoIterator)]
pub struct GenericParameters<T: std::fmt::Display + AsRef<str>>(Vec<GenericParameter<T>>);

impl<T> std::fmt::Display for GenericParameters<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", join(&self.0, ";"))
    }
}

impl<T> PartialEq for GenericParameters<T>
where
    T: std::fmt::Display + AsRef<str> + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        compare_vectors(self.0.deref(), other.0.deref())
    }
}

/// Representation of a generic parameter.
#[derive(Clone, Debug, Eq)]
pub struct GenericParameter<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    key: T,
    value: Option<WrappedString<T>>,
}

impl<T> GenericParameter<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    /// Create a `GenericParameter`.
    pub fn new(key: T, value: Option<WrappedString<T>>) -> Self {
        Self { key, value }
    }

    /// Get the key of the generic parameter.
    pub fn key(&self) -> &str {
        self.key.as_ref()
    }

    /// Get the value of the generic parameter.
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

impl<T> std::fmt::Display for GenericParameter<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.key().to_ascii_lowercase(),
            if self.value().is_some() { "=" } else { "" },
            self.value().unwrap_or_default().to_ascii_lowercase()
        )
    }
}

impl<T> PartialEq for GenericParameter<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    fn eq(&self, other: &GenericParameter<T>) -> bool {
        self.key().eq_ignore_ascii_case(other.key())
            && self.value().map(|v| v.to_ascii_lowercase())
                == other.value().map(|v| v.to_ascii_lowercase())
    }
}

impl<T> PartialOrd for GenericParameter<T>
where
    T: std::fmt::Display + AsRef<str> + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for GenericParameter<T>
where
    T: std::fmt::Display + AsRef<str> + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .key()
            .to_ascii_lowercase()
            .cmp(&other.key().to_ascii_lowercase())
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

impl<T> Hash for GenericParameter<T>
where
    T: std::fmt::Display + AsRef<str>,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        self.value().map(|v| v.to_ascii_lowercase()).hash(state);
    }
}

pub(crate) mod parser {
    use crate::common::wrapped_string::WrappedString;
    use crate::parser::{equal, quoted_string, token, ParserResult};
    use crate::uris::host::parser::host;
    use crate::{GenericParameter, TokenString};
    use nom::{
        branch::alt,
        combinator::{map, opt, recognize},
        error::context,
        sequence::{pair, preceded},
    };

    fn gen_value(input: &str) -> ParserResult<&str, WrappedString<TokenString>> {
        context(
            "gen_value",
            alt((
                map(token, WrappedString::new_not_wrapped),
                map(recognize(host), |v| {
                    WrappedString::new_not_wrapped(TokenString::new(v))
                }),
                quoted_string,
            )),
        )(input)
    }

    pub(crate) fn generic_param(input: &str) -> ParserResult<&str, GenericParameter<TokenString>> {
        context(
            "generic_param",
            map(
                pair(token, opt(preceded(equal, gen_value))),
                |(name, value)| GenericParameter::new(name, value),
            ),
        )(input)
    }
}
