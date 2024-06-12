use derive_more::IsVariant;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::common::message_qop::MessageQop;
use crate::common::wrapped_string::WrappedString;

/// Representation of the list of authentication infos from an
/// `AuthenticationInfoHeader`.
///
/// This is usable as an iterator.
pub type AuthenticationInfos = HeaderValueCollection<AuthenticationInfo>;

/// Representation of an info from an `AuthenticationInfoHeader`.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
#[non_exhaustive]
pub enum AuthenticationInfo {
    /// A `nextnonce` authentication info.
    NextNonce(WrappedString),
    /// A `qop` authentication info.
    Qop(MessageQop),
    /// A `rspauth` authentication info.
    ResponseAuth(WrappedString),
    /// A `cnonce` authentication info.
    CNonce(WrappedString),
    /// A `nonce` authentication info.
    NonceCount(WrappedString),
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
