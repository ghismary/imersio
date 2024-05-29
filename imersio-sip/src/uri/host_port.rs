use std::{hash::Hash, num::NonZeroU16};

use partial_eq_refs::PartialEqRefs;

/// Representation of a hostport of a SIP URI.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct HostPort {
    host: String,
    port: Option<NonZeroU16>,
}

impl HostPort {
    pub(crate) fn new<S: Into<String>>(host: S, port: Option<NonZeroU16>) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    /// Get the host part of the `HostPort`.
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Get the port part of the `HostPort`.
    pub fn get_port(&self) -> Option<NonZeroU16> {
        self.port
    }
}

impl std::fmt::Display for HostPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.host,
            if self.port.is_some() { ":" } else { "" },
            self.port.map(|p| format!("{p}")).unwrap_or_default()
        )
    }
}

impl Default for HostPort {
    fn default() -> Self {
        HostPort {
            host: "localhost".to_string(),
            port: None,
        }
    }
}

impl PartialEq for HostPort {
    fn eq(&self, other: &Self) -> bool {
        self.host.eq_ignore_ascii_case(&other.host) && self.port == other.port
    }
}

impl Hash for HostPort {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.host.to_ascii_lowercase().hash(state);
        self.port.hash(state);
    }
}
