use derive_more::IsVariant;
use std::hash::Hash;
use std::net::IpAddr;

use partial_eq_refs::PartialEqRefs;

/// Representation of a hostport of a SIP URI.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
pub enum Host {
    /// A hostname
    Name(String),
    /// An Ip address.
    Ip(IpAddr),
}

impl Host {
    /// Get the name of the `Host` is it is one.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Name(name) => Some(name.as_str()),
            _ => None,
        }
    }

    /// Get the ip of the `Host` if it is one.
    pub fn ip(&self) -> Option<&IpAddr> {
        match self {
            Self::Ip(ip) => Some(ip),
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
                Self::Ip(ip) => ip.to_string(),
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
            (Self::Ip(a), Self::Ip(b)) => a == b,
            _ => false,
        }
    }
}

impl Hash for Host {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Name(name) => name.to_ascii_lowercase().hash(state),
            Self::Ip(ip) => ip.hash(state),
        }
    }
}
