use std::ops::Deref;

use crate::Algorithm;
use crate::MessageQop;
use crate::Uri;
use crate::utils::compare_vectors;
use crate::{AuthParameter, AuthParameters};

/// Representation of the credentials from an `AuthorizationHeader` or a `ProxyAuthorizationHeader`.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum Credentials {
    /// The Digest authentication scheme.
    ///
    /// [[RFC3261, Section 22.4](https://datatracker.ietf.org/doc/html/rfc3261#section-22.4)]
    Digest(AuthParameters),
    /// Any other extension authentication scheme.
    Other(String, AuthParameters),
}

impl Credentials {
    /// Tell whether the credentials contain the given authorization parameter key.
    pub fn contains(&self, key: &str) -> bool {
        self.parameters().iter().any(|p| p.key() == key)
    }

    /// Get the `AuthParam` corresponding to the given authorization parameter key.
    pub fn get(&self, key: &str) -> Option<&AuthParameter> {
        self.parameters().iter().find(|p| p.key() == key)
    }

    /// Get the scheme of the credentials.
    pub fn scheme(&self) -> &str {
        match self {
            Self::Digest(_) => "Digest",
            Self::Other(scheme, _) => scheme,
        }
    }

    /// Get a reference to the `AuthParam`s in the credentials.
    pub fn parameters(&self) -> &AuthParameters {
        match self {
            Self::Digest(params) => params,
            Self::Other(_, params) => params,
        }
    }

    /// Tell whether the credentials contain an `algorithm` value.
    pub fn has_algorithm(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::Algorithm(_))),
            _ => false,
        }
    }

    /// Get the `algorithm` value from the credentials.
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

    /// Tell whether the credentials contain a `uri` value.
    pub fn has_digest_uri(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::DigestUri(_))),
            _ => false,
        }
    }

    /// Get the `uri` value from the Authorization header.
    pub fn digest_uri(&self) -> Option<&Uri> {
        match self {
            Self::Digest(params) => params
                .iter()
                .find(|param| matches!(param, AuthParameter::DigestUri(_)))
                .and_then(|param| {
                    if let AuthParameter::DigestUri(value) = param {
                        Some(value)
                    } else {
                        None
                    }
                }),
            _ => None,
        }
    }

    /// Tell whether the credentials contain a `qop` value.
    pub fn has_qop(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::Qop(_))),
            _ => false,
        }
    }

    /// Get the `qop` value from the credentials.
    pub fn qop(&self) -> Option<&MessageQop> {
        match self {
            Self::Digest(params) => params
                .iter()
                .find(|param| matches!(param, AuthParameter::Qop(_)))
                .and_then(|param| {
                    if let AuthParameter::Qop(value) = param {
                        Some(value)
                    } else {
                        None
                    }
                }),
            _ => None,
        }
    }
}

impl std::fmt::Display for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.scheme(), self.parameters())
    }
}

impl PartialEq for Credentials {
    fn eq(&self, other: &Self) -> bool {
        if !self.scheme().eq_ignore_ascii_case(other.scheme()) {
            false
        } else {
            compare_vectors(self.parameters().deref(), other.parameters().deref())
        }
    }
}

macro_rules! credentials {
    (
        $(
            ($token:ident, $has_token:ident, $enum_name:ident),
        )+
    ) => {
        impl Credentials {
            $(
                /// Tell whether the credentials contain a `$token` value.
                pub fn $has_token(&self) -> bool {
                    match self {
                        Self::Digest(params) => params.iter().any(|param| matches!(param, AuthParameter::$enum_name(_))),
                        _ => false
                    }
                }

                /// Get the `$token` value from the credentials.
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

credentials! {
    (username, has_username, Username),
    (realm, has_realm, Realm),
    (nonce, has_nonce, Nonce),
    (dresponse, has_dresponse, DResponse),
    (cnonce, has_cnonce, CNonce),
    (opaque, has_opaque, Opaque),
    (nonce_count, has_nonce_count, NonceCount),
}

pub(crate) mod parser {
    use nom::{
        Parser,
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{cut, map},
        error::context,
        sequence::separated_pair,
    };

    use crate::{
        Credentials,
        common::auth_parameter::parser::{auth_params, auth_scheme, digest_response},
        parser::{ParserResult, lws},
    };

    fn digest_credentials(input: &str) -> ParserResult<&str, Credentials> {
        context(
            "digest_credentials",
            map(
                separated_pair(tag_no_case("Digest"), lws, cut(digest_response)),
                |(_, params)| Credentials::Digest(params),
            ),
        )
        .parse(input)
    }

    fn other_response(input: &str) -> ParserResult<&str, Credentials> {
        context(
            "other_response",
            map(
                separated_pair(auth_scheme, lws, auth_params),
                |(scheme, params)| Credentials::Other(scheme.to_string(), params),
            ),
        )
        .parse(input)
    }

    pub(crate) fn credentials(input: &str) -> ParserResult<&str, Credentials> {
        context("credentials", alt((digest_credentials, other_response))).parse(input)
    }
}
