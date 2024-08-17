use derive_more::IsVariant;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::common::wrapped_string::WrappedString;
use crate::Algorithm;
use crate::AuthenticationInfo;
use crate::DomainUris;
use crate::Stale;
use crate::{MessageQop, MessageQops};
use crate::{SipError, Uri};

/// Representation of a list of authentication parameters from an `AuthorizationHeader` or a
/// `ProxyAuthenticateHeader`.
///
/// This is usable as an iterator.
pub type AuthParameters = ValueCollection<AuthParameter>;

/// Representation of the authentication parameters used in an `AuthorizationHeader` or in a
/// `ProxyAuthenticateHeader`.
#[derive(Clone, Debug, Eq, IsVariant)]
pub enum AuthParameter {
    /// A `username` parameter.
    Username(WrappedString),
    /// A `realm` parameter.
    Realm(WrappedString),
    /// A `nonce` parameter.
    Nonce(WrappedString),
    /// An `uri` parameter.
    DigestUri(Uri),
    /// A `response` parameter.
    DResponse(WrappedString),
    /// An `algorithm` parameter.
    Algorithm(Algorithm),
    /// A `cnonce` parameter.
    CNonce(WrappedString),
    /// An `opaque` parameter.
    Opaque(WrappedString),
    /// A `qop` parameter with a single value in an `AuthorizationHeader`.
    Qop(MessageQop),
    /// A `nc` parameter.
    NonceCount(WrappedString),
    /// A `domain` parameter in a `ProxyAuthenticateHeader`.
    Domain(DomainUris),
    /// A `stale` parameter in a `ProxyAuthenticateHeader`.
    Stale(Stale),
    /// A `qop` parameter in a `ProxyAuthenticateHeader`.
    QopOptions(MessageQops),
    /// Any other parameter.
    Other(String, WrappedString),
}

impl AuthParameter {
    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Username(_) => "username",
            Self::Realm(_) => "realm",
            Self::Nonce(_) => "nonce",
            Self::DigestUri(_) => "uri",
            Self::DResponse(_) => "response",
            Self::Algorithm(_) => "algorithm",
            Self::CNonce(_) => "cnonce",
            Self::Opaque(_) => "opaque",
            Self::Qop(_) => "qop",
            Self::NonceCount(_) => "nc",
            Self::Domain(_) => "domain",
            Self::Stale(_) => "stale",
            Self::QopOptions(_) => "qop",
            Self::Other(key, _) => key,
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> String {
        match self {
            Self::Username(value) => value.value(),
            Self::Realm(value) => value.value(),
            Self::Nonce(value) => value.value(),
            Self::DigestUri(value) => value.to_string(),
            Self::DResponse(value) => value.value(),
            Self::Algorithm(value) => value.value().into(),
            Self::CNonce(value) => value.value(),
            Self::Opaque(value) => value.value(),
            Self::Qop(value) => value.value().into(),
            Self::NonceCount(value) => value.value(),
            Self::Domain(value) => value.to_string(),
            Self::Stale(value) => value.to_string(),
            Self::QopOptions(value) => value.to_string(),
            Self::Other(_, value) => value.value(),
        }
    }
}

impl std::fmt::Display for AuthParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (key, value) = match self {
            Self::Username(value) => ("username".into(), value.to_string()),
            Self::Realm(value) => ("realm".into(), value.to_string()),
            Self::Nonce(value) => ("nonce".into(), value.to_string()),
            Self::DigestUri(value) => ("uri".into(), format!("\"{value}\"")),
            Self::DResponse(value) => ("response".into(), value.to_string()),
            Self::Algorithm(value) => ("algorithm".into(), value.to_string()),
            Self::CNonce(value) => ("cnonce".into(), value.to_string()),
            Self::Opaque(value) => ("opaque".into(), value.to_string()),
            Self::Qop(value) => ("qop".into(), value.to_string()),
            Self::NonceCount(value) => ("nc".into(), value.to_string()),
            Self::Domain(value) => ("domain".into(), value.to_string()),
            Self::Stale(value) => ("stale".into(), value.to_string()),
            Self::QopOptions(value) => ("qop".into(), value.to_string()),
            Self::Other(key, value) => (key.clone(), value.to_string()),
        };
        write!(f, "{}={}", key, value)
    }
}

impl PartialEq for AuthParameter {
    fn eq(&self, other: &AuthParameter) -> bool {
        match (self, other) {
            (Self::Username(a), Self::Username(b))
            | (Self::Realm(a), Self::Realm(b))
            | (Self::Nonce(a), Self::Nonce(b))
            | (Self::DResponse(a), Self::DResponse(b))
            | (Self::CNonce(a), Self::CNonce(b))
            | (Self::Opaque(a), Self::Opaque(b))
            | (Self::NonceCount(a), Self::NonceCount(b)) => a == b,
            (Self::DigestUri(a), Self::DigestUri(b)) => a == b,
            (Self::Algorithm(a), Self::Algorithm(b)) => a == b,
            (Self::Qop(a), Self::Qop(b)) => a == b,
            (Self::Domain(a), Self::Domain(b)) => a == b,
            (Self::Stale(a), Self::Stale(b)) => a == b,
            (Self::QopOptions(a), Self::QopOptions(b)) => a == b,
            (Self::Other(akey, avalue), Self::Other(bkey, bvalue)) => {
                akey.eq_ignore_ascii_case(bkey) && avalue == bvalue
            }
            _ => false,
        }
    }
}

impl PartialOrd for AuthParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AuthParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl Hash for AuthParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        match self {
            Self::Username(value)
            | Self::Realm(value)
            | Self::Nonce(value)
            | Self::DResponse(value)
            | Self::CNonce(value)
            | Self::Opaque(value)
            | Self::NonceCount(value) => value.hash(state),
            Self::DigestUri(value) => value.hash(state),
            Self::Algorithm(value) => value.hash(state),
            Self::Qop(value) => value.hash(state),
            Self::Domain(value) => value.hash(state),
            Self::Stale(value) => value.hash(state),
            Self::QopOptions(value) => value.hash(state),
            Self::Other(_, value) => value.to_ascii_lowercase().hash(state),
        }
    }
}

impl TryFrom<AuthenticationInfo> for AuthParameter {
    type Error = SipError;

    fn try_from(value: AuthenticationInfo) -> Result<Self, Self::Error> {
        match value {
            AuthenticationInfo::CNonce(value) => Ok(AuthParameter::CNonce(value)),
            AuthenticationInfo::Qop(value) => Ok(AuthParameter::Qop(value)),
            AuthenticationInfo::NonceCount(value) => Ok(AuthParameter::NonceCount(value)),
            _ => Err(SipError::FailedConvertingAInfoToAuthParam),
        }
    }
}

pub(crate) mod parser {
    use crate::common::authentication_info::parser::{
        cnonce, message_qop, nonce_count, nonce_value,
    };
    use crate::common::wrapped_string::WrappedString;
    use crate::parser::{comma, equal, ldquot, lhex, quoted_string, rdquot, token, ParserResult};
    use crate::uris::uri::parser::request_uri;
    use crate::{Algorithm, AuthParameter, AuthParameters, Uri};
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{cut, map, recognize},
        error::context,
        multi::{many_m_n, separated_list1},
        sequence::{delimited, separated_pair},
    };

    #[inline]
    fn username_value(input: &str) -> ParserResult<&str, WrappedString> {
        quoted_string(input)
    }

    fn username(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "username",
            map(
                separated_pair(tag_no_case("username"), equal, cut(username_value)),
                |(_, value)| AuthParameter::Username(value),
            ),
        )(input)
    }

    #[inline]
    fn realm_value(input: &str) -> ParserResult<&str, WrappedString> {
        quoted_string(input)
    }

    pub(crate) fn realm(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "realm",
            map(
                separated_pair(tag_no_case("realm"), equal, cut(realm_value)),
                |(_, value)| AuthParameter::Realm(value),
            ),
        )(input)
    }

    pub(crate) fn nonce(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "nonce",
            map(
                separated_pair(tag_no_case("nonce"), equal, cut(nonce_value)),
                |(_, value)| AuthParameter::Nonce(value),
            ),
        )(input)
    }

    fn digest_uri_value(input: &str) -> ParserResult<&str, Uri> {
        delimited(ldquot, request_uri, rdquot)(input)
    }

    fn digest_uri(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "digest_uri",
            map(
                separated_pair(tag_no_case("uri"), equal, cut(digest_uri_value)),
                |(_, value)| AuthParameter::DigestUri(value),
            ),
        )(input)
    }

    fn request_digest(input: &str) -> ParserResult<&str, WrappedString> {
        context(
            "request_digest",
            map(
                delimited(ldquot, recognize(many_m_n(32, 32, lhex)), rdquot),
                WrappedString::new_quoted,
            ),
        )(input)
    }

    fn dresponse(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "dresponse",
            map(
                separated_pair(tag_no_case("response"), equal, cut(request_digest)),
                |(_, value)| AuthParameter::DResponse(value),
            ),
        )(input)
    }

    pub(crate) fn algorithm(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "algorithm",
            map(
                separated_pair(
                    tag_no_case("algorithm"),
                    equal,
                    cut(alt((tag_no_case("MD5"), tag_no_case("MD5-sess"), token))),
                ),
                |(_, value)| AuthParameter::Algorithm(Algorithm::new(value)),
            ),
        )(input)
    }

    pub(crate) fn opaque(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "opaque",
            map(
                separated_pair(tag_no_case("opaque"), equal, cut(quoted_string)),
                |(_, value)| AuthParameter::Opaque(value),
            ),
        )(input)
    }

    #[inline]
    fn auth_param_name(input: &str) -> ParserResult<&str, &str> {
        token(input)
    }

    pub(crate) fn auth_param(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "auth_param",
            map(
                separated_pair(
                    auth_param_name,
                    equal,
                    alt((map(token, WrappedString::new_not_wrapped), quoted_string)),
                ),
                |(key, value)| AuthParameter::Other(key.to_string(), value),
            ),
        )(input)
    }

    fn dig_resp(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "dig_resp",
            alt((
                username,
                realm,
                nonce,
                digest_uri,
                dresponse,
                algorithm,
                map(cnonce, |ainfo| ainfo.try_into().unwrap()),
                opaque,
                map(message_qop, |ainfo| ainfo.try_into().unwrap()),
                map(nonce_count, |ainfo| ainfo.try_into().unwrap()),
                auth_param,
            )),
        )(input)
    }

    pub(crate) fn digest_response(input: &str) -> ParserResult<&str, AuthParameters> {
        context(
            "digest_response",
            map(separated_list1(comma, dig_resp), Into::into),
        )(input)
    }

    #[inline]
    pub(crate) fn auth_scheme(input: &str) -> ParserResult<&str, &str> {
        token(input)
    }

    pub(crate) fn auth_params(input: &str) -> ParserResult<&str, AuthParameters> {
        context(
            "auth_params",
            map(separated_list1(comma, auth_param), Into::into),
        )(input)
    }
}
