#![allow(missing_docs)]

use derive_more::IsVariant;
use partial_eq_refs::PartialEqRefs;
use std::ops::Deref;

use crate::utils::compare_vectors;
use crate::Algorithm;
use crate::DomainUris;
use crate::MessageQops;
use crate::Stale;
use crate::{AuthParameter, AuthParameters};

/// Representation of the challenge from an `ProxyAuthenticateHeader`.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
pub enum Challenge {
    /// The Digest authentication scheme.
    ///
    /// [[RFC3261, Section 22.4](https://datatracker.ietf.org/doc/html/rfc3261#section-22.4)]
    Digest(AuthParameters),
    /// Any other extension authentication scheme.
    Other(String, AuthParameters),
}

impl Challenge {
    /// Tell whether Proxy-Authenticate header contains the given authorization parameter key.
    pub fn contains(&self, key: &str) -> bool {
        self.parameters().iter().any(|p| p.key() == key)
    }

    /// Get the `AuthParam` corresponding to the given authorization parameter key.
    pub fn get(&self, key: &str) -> Option<&AuthParameter> {
        self.parameters().iter().find(|p| p.key() == key)
    }

    /// Get the scheme of the `Challenge`.
    pub fn scheme(&self) -> &str {
        match self {
            Self::Digest(_) => "Digest",
            Self::Other(scheme, _) => scheme,
        }
    }

    /// Get a reference to the `AuthParam`s in the `Challenge`.
    pub fn parameters(&self) -> &AuthParameters {
        match self {
            Self::Digest(params) => params,
            Self::Other(_, params) => params,
        }
    }

    /// Tell whether the Proxy-Authenticate header contains an `algorithm` value.
    pub fn has_algorithm(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::Algorithm(_))),
            _ => false,
        }
    }

    /// Get the `algorithm` value from the Authorization header.
    pub fn algorithm(&self) -> Option<&Algorithm> {
        match self {
            Self::Digest(params) => params
                .iter()
                .find(|param| matches!(param, AuthParameter::Algorithm(_)))
                .and_then(|param| {
                    if let AuthParameter::Algorithm(value) = param {
                        Some(value)
                    } else {
                        None
                    }
                }),
            _ => None,
        }
    }

    /// Tell whether the Proxy-Authenticate header contains a `domain` value.
    pub fn has_domain(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::Domain(_))),
            _ => false,
        }
    }

    /// Get the `domain` value from the Authorization header.
    pub fn domain(&self) -> Option<&DomainUris> {
        match self {
            Self::Digest(params) => params
                .iter()
                .find(|param| matches!(param, AuthParameter::Domain(_)))
                .and_then(|param| {
                    if let AuthParameter::Domain(value) = param {
                        Some(value)
                    } else {
                        None
                    }
                }),
            _ => None,
        }
    }

    /// Tell whether the Proxy-Authenticate header contains a `stale` value.
    pub fn has_stale(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::Stale(_))),
            _ => false,
        }
    }

    /// Get the `stale` value from the Proxy-Authenticate header.
    pub fn stale(&self) -> Option<&Stale> {
        match self {
            Self::Digest(params) => params
                .iter()
                .find(|param| matches!(param, AuthParameter::Stale(_)))
                .and_then(|param| {
                    if let AuthParameter::Stale(value) = param {
                        Some(value)
                    } else {
                        None
                    }
                }),
            _ => None,
        }
    }

    /// Tell whether the Proxy-Authenticate header contains a `qop` value.
    pub fn has_qop(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::QopOptions(_))),
            _ => false,
        }
    }

    /// Get the `qop` value from the Proxy-Authenticate header.
    pub fn qop(&self) -> Option<&MessageQops> {
        match self {
            Self::Digest(params) => params
                .iter()
                .find(|param| matches!(param, AuthParameter::QopOptions(_)))
                .and_then(|param| {
                    if let AuthParameter::QopOptions(value) = param {
                        Some(value)
                    } else {
                        None
                    }
                }),
            _ => None,
        }
    }
}

impl std::fmt::Display for Challenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.scheme(), self.parameters())
    }
}

impl PartialEq for Challenge {
    fn eq(&self, other: &Self) -> bool {
        if !self.scheme().eq_ignore_ascii_case(other.scheme()) {
            false
        } else {
            compare_vectors(self.parameters().deref(), other.parameters().deref())
        }
    }
}

macro_rules! challenge {
    (
        $(
            ($token:ident, $has_token:ident, $enum_name:ident),
        )+
    ) => {
        impl Challenge {
            $(
                /// Tell whether the Authorization header contains a `$token` value.
                pub fn $has_token(&self) -> bool {
                    match self {
                        Self::Digest(params) => params.iter().any(|param| matches!(param, AuthParameter::$enum_name(_))),
                        _ => false
                    }
                }

                /// Get the `$token` value from the Authorization header.
                pub fn $token(&self) -> Option<&str> {
                    match self {
                        Self::Digest(params) => params
                        .iter()
                        .find(|param| matches!(param, AuthParameter::$enum_name(_)))
                        .map(|param| {
                            if let AuthParameter::$enum_name(value) = param {
                                value
                            } else {
                                ""
                            }
                        }),
                        _ => None
                    }
                }
            )+
        }
    }
}

challenge! {
    (realm, has_realm, Realm),
    (nonce, has_nonce, Nonce),
    (opaque, has_opaque, Opaque),
}

pub(crate) mod parser {
    use crate::common::auth_parameter::parser::{
        algorithm, auth_param, auth_scheme, nonce, opaque, realm,
    };
    use crate::common::authentication_info::parser::qop_value;
    use crate::parser::{comma, equal, ldquot, lws, param, pchar, rdquot, sp, ParserResult};
    use crate::uris::parser::request_uri;
    use crate::{AuthParameter, Challenge, DomainUri, MessageQop, Stale};
    use nom::{
        branch::alt,
        bytes::complete::{tag, tag_no_case},
        combinator::{consumed, cut, map, recognize, value},
        error::context,
        multi::{many0, many1, separated_list1},
        sequence::{delimited, pair, preceded, separated_pair, tuple},
    };

    fn other_challenge(input: &str) -> ParserResult<&str, Challenge> {
        context(
            "other_challenge",
            map(
                separated_pair(auth_scheme, lws, separated_list1(comma, auth_param)),
                |(scheme, auth_params)| Challenge::Other(scheme.to_string(), auth_params.into()),
            ),
        )(input)
    }

    fn segment(input: &str) -> ParserResult<&str, &str> {
        context(
            "segment",
            recognize(pair(many0(pchar), many0(preceded(tag(";"), param)))),
        )(input)
    }

    fn path_segments(input: &str) -> ParserResult<&str, &str> {
        context(
            "path_segments",
            recognize(pair(segment, many0(preceded(tag("/"), segment)))),
        )(input)
    }

    fn abs_path(input: &str) -> ParserResult<&str, &str> {
        context("abs_path", recognize(pair(tag("/"), path_segments)))(input)
    }

    fn uri(input: &str) -> ParserResult<&str, DomainUri> {
        context(
            "uri",
            alt((
                map(request_uri, DomainUri::Uri),
                map(abs_path, |path| DomainUri::AbsPath(path.to_string())),
            )),
        )(input)
    }

    fn domain_value(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "domain_value",
            delimited(
                ldquot,
                map(separated_list1(many1(sp), uri), |uris| {
                    AuthParameter::Domain(uris.into())
                }),
                rdquot,
            ),
        )(input)
    }

    fn domain(input: &str) -> ParserResult<&str, AuthParameter> {
        map(
            tuple((tag_no_case("domain"), equal, cut(domain_value))),
            |(_, _, domain)| domain,
        )(input)
    }

    fn stale(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "stale",
            map(
                separated_pair(
                    tag_no_case("stale"),
                    equal,
                    cut(map(
                        consumed(alt((
                            value(true, tag_no_case("true")),
                            value(false, tag_no_case("false")),
                        ))),
                        |(s, v)| AuthParameter::Stale(Stale::new(s, v)),
                    )),
                ),
                |(_, stale)| stale,
            ),
        )(input)
    }

    fn qop_options(input: &str) -> ParserResult<&str, AuthParameter> {
        context(
            "qop_options",
            map(
                separated_pair(
                    tag_no_case("qop"),
                    equal,
                    cut(delimited(
                        ldquot,
                        separated_list1(tag(","), qop_value),
                        rdquot,
                    )),
                ),
                |(_, values)| {
                    AuthParameter::QopOptions(
                        values
                            .iter()
                            .map(|v| MessageQop::new(v.to_string()))
                            .collect::<Vec<MessageQop>>()
                            .into(),
                    )
                },
            ),
        )(input)
    }

    fn digest_cln(input: &str) -> ParserResult<&str, AuthParameter> {
        alt((
            realm,
            domain,
            nonce,
            opaque,
            stale,
            algorithm,
            qop_options,
            auth_param,
        ))(input)
    }

    pub(crate) fn challenge(input: &str) -> ParserResult<&str, Challenge> {
        alt((
            map(
                separated_pair(
                    tag_no_case("Digest"),
                    lws,
                    cut(separated_list1(comma, digest_cln)),
                ),
                |(_, auth_params)| Challenge::Digest(auth_params.into()),
            ),
            other_challenge,
        ))(input)
    }
}
