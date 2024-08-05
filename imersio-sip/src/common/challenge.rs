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
