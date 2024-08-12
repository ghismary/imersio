#![allow(missing_docs)]

use derive_more::IsVariant;
use std::hash::Hash;
use std::net::{Ipv4Addr, Ipv6Addr};

use partial_eq_refs::PartialEqRefs;

/// Representation of a hostport of a SIP URI.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
pub enum Host {
    /// A hostname
    Name(String),
    /// An Ipv4 address.
    Ipv4(Ipv4Addr),
    /// An Ipv6 address.
    Ipv6(Ipv6Addr),
}

impl Host {
    /// Get the name of the `Host` is it is one.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Name(name) => Some(name.as_str()),
            _ => None,
        }
    }

    /// Get the ipv4 of the `Host` if it is one.
    pub fn ipv4(&self) -> Option<&Ipv4Addr> {
        match self {
            Self::Ipv4(ipv4) => Some(ipv4),
            _ => None,
        }
    }

    /// Get the ipv6 of the `Host` if it is one.
    pub fn ipv6(&self) -> Option<&Ipv6Addr> {
        match self {
            Self::Ipv6(ipv6) => Some(ipv6),
            _ => None,
        }
    }
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Name(name) => name.clone(),
                Self::Ipv4(ipv4) => ipv4.to_string(),
                Self::Ipv6(ipv6) => ipv6.to_string(),
            }
        )
    }
}

impl Default for Host {
    fn default() -> Self {
        Host::Name("localhost".to_string())
    }
}

impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Name(a), Self::Name(b)) => a.eq_ignore_ascii_case(b),
            (Self::Ipv4(a), Self::Ipv4(b)) => a == b,
            (Self::Ipv6(a), Self::Ipv6(b)) => a == b,
            _ => false,
        }
    }
}

impl Hash for Host {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Name(name) => name.to_ascii_lowercase().hash(state),
            Self::Ipv4(ipv4) => ipv4.hash(state),
            Self::Ipv6(ipv6) => ipv6.hash(state),
        }
    }
}
