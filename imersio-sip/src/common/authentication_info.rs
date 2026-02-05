use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::common::wrapped_string::WrappedString;
use crate::{MessageQop, TokenString};

/// Representation of the list of authentication infos from an
/// `AuthenticationInfoHeader`.
///
/// This is usable as an iterator.
pub type AuthenticationInfos = ValueCollection<AuthenticationInfo>;

/// Representation of an info from an `AuthenticationInfoHeader`.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
#[non_exhaustive]
pub enum AuthenticationInfo {
    /// A `nextnonce` authentication info.
    NextNonce(WrappedString<TokenString>),
    /// A `qop` authentication info.
    Qop(MessageQop),
    /// A `rspauth` authentication info.
    ResponseAuth(WrappedString<TokenString>),
    /// A `cnonce` authentication info.
    CNonce(WrappedString<TokenString>),
    /// A `nonce` authentication info.
    NonceCount(WrappedString<TokenString>),
}

impl AuthenticationInfo {
    /// Get the key of the authentication info.
    pub fn key(&self) -> &str {
        match self {
            Self::NextNonce(_) => "nextnonce",
            Self::Qop(_) => "qop",
            Self::ResponseAuth(_) => "rspauth",
            Self::CNonce(_) => "cnonce",
            Self::NonceCount(_) => "nc",
        }
    }

    /// Get the value of the authentication info.
    pub fn value(&self) -> &str {
        match self {
            Self::NextNonce(value) | Self::ResponseAuth(value) | Self::CNonce(value) => {
                value.as_ref()
            }
            Self::NonceCount(value) => value,
            Self::Qop(value) => value.value(),
        }
    }
}

impl std::fmt::Display for AuthenticationInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (key, value) = match self {
            Self::NextNonce(value) => ("nextnonce", value.to_string()),
            Self::Qop(value) => ("qop", value.to_string()),
            Self::ResponseAuth(value) => ("rspauth", value.to_string()),
            Self::CNonce(value) => ("cnonce", value.to_string()),
            Self::NonceCount(value) => ("nc", value.to_string()),
        };
        write!(f, "{}={}", key, value)
    }
}

impl PartialEq for AuthenticationInfo {
    fn eq(&self, other: &AuthenticationInfo) -> bool {
        match (self, other) {
            (Self::NextNonce(a), Self::NextNonce(b))
            | (Self::ResponseAuth(a), Self::ResponseAuth(b))
            | (Self::CNonce(a), Self::CNonce(b))
            | (Self::NonceCount(a), Self::NonceCount(b)) => a == b,
            (Self::Qop(a), Self::Qop(b)) => a == b,
            _ => false,
        }
    }
}

impl Hash for AuthenticationInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        match self {
            Self::NextNonce(value)
            | Self::ResponseAuth(value)
            | Self::CNonce(value)
            | Self::NonceCount(value) => value.hash(state),
            Self::Qop(value) => value.value().to_ascii_lowercase().hash(state),
        }
    }
}

pub(crate) mod parser {
    use nom::{
        Parser,
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{cut, map, recognize},
        error::context,
        multi::{count, many0},
        sequence::{delimited, separated_pair},
    };

    use crate::{
        AuthenticationInfo, MessageQop, TokenString,
        common::wrapped_string::WrappedString,
        parser::{ParserResult, equal, ldquot, lhex, quoted_string, rdquot, token},
    };

    #[inline]
    pub(crate) fn nonce_value(input: &str) -> ParserResult<&str, WrappedString<TokenString>> {
        quoted_string(input)
    }

    fn nextnonce(input: &str) -> ParserResult<&str, AuthenticationInfo> {
        context(
            "nextnonce",
            map(
                separated_pair(tag_no_case("nextnonce"), equal, nonce_value),
                |(_, value)| AuthenticationInfo::NextNonce(value),
            ),
        )
        .parse(input)
    }

    pub(crate) fn qop_value(input: &str) -> ParserResult<&str, TokenString> {
        alt((
            map(tag_no_case("auth-int"), TokenString::new),
            map(tag_no_case("auth"), TokenString::new),
            token,
        ))
        .parse(input)
    }

    pub(crate) fn message_qop(input: &str) -> ParserResult<&str, AuthenticationInfo> {
        context(
            "message_qop",
            map(
                separated_pair(tag_no_case("qop"), equal, cut(qop_value)),
                |(_, value)| AuthenticationInfo::Qop(MessageQop::new(value)),
            ),
        )
        .parse(input)
    }

    fn response_digest(input: &str) -> ParserResult<&str, WrappedString<TokenString>> {
        map(delimited(ldquot, many0(lhex), rdquot), |digits| {
            WrappedString::new_quoted(
                digits
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<String>>()
                    .join(""),
            )
        })
        .parse(input)
    }

    fn response_auth(input: &str) -> ParserResult<&str, AuthenticationInfo> {
        context(
            "response_auth",
            map(
                separated_pair(tag_no_case("rspauth"), equal, response_digest),
                |(_, value)| AuthenticationInfo::ResponseAuth(value),
            ),
        )
        .parse(input)
    }

    #[inline]
    fn cnonce_value(input: &str) -> ParserResult<&str, WrappedString<TokenString>> {
        nonce_value(input)
    }

    pub(crate) fn cnonce(input: &str) -> ParserResult<&str, AuthenticationInfo> {
        context(
            "cnonce",
            map(
                separated_pair(tag_no_case("cnonce"), equal, cut(cnonce_value)),
                |(_, value)| AuthenticationInfo::CNonce(value),
            ),
        )
        .parse(input)
    }

    fn nc_value(input: &str) -> ParserResult<&str, WrappedString<TokenString>> {
        map(recognize(count(lhex, 8)), |digits| {
            WrappedString::new_not_wrapped(TokenString::new(digits))
        })
        .parse(input)
    }

    pub(crate) fn nonce_count(input: &str) -> ParserResult<&str, AuthenticationInfo> {
        context(
            "nonce_count",
            map(
                separated_pair(tag_no_case("nc"), equal, cut(nc_value)),
                |(_, value)| AuthenticationInfo::NonceCount(value),
            ),
        )
        .parse(input)
    }

    pub(crate) fn ainfo(input: &str) -> ParserResult<&str, AuthenticationInfo> {
        alt((nextnonce, message_qop, response_auth, cnonce, nonce_count)).parse(input)
    }
}
