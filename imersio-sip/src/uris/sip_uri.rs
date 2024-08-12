//! TODO

use partial_eq_refs::PartialEqRefs;

use crate::{Host, UriHeaders, UriParameters, UriScheme, UserInfo};

/// Representation of a SIP URI.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, PartialEqRefs)]
pub struct SipUri {
    scheme: UriScheme,
    userinfo: Option<UserInfo>,
    host: Host,
    port: Option<u16>,
    parameters: UriParameters,
    headers: UriHeaders,
}

impl SipUri {
    pub(crate) fn new(
        scheme: UriScheme,
        userinfo: Option<UserInfo>,
        host: Host,
        port: Option<u16>,
        parameters: UriParameters,
        headers: UriHeaders,
    ) -> Self {
        Self {
            scheme,
            userinfo,
            host,
            port,
            parameters,
            headers,
        }
    }

    /// Get a reference to the `UriScheme` of the sip uri.
    pub fn scheme(&self) -> &UriScheme {
        &self.scheme
    }

    /// Get a reference to the `UserInfo` of the sip uri.
    pub fn userinfo(&self) -> Option<&UserInfo> {
        self.userinfo.as_ref()
    }

    /// Get a reference to the `Host` of the sip uri.
    pub fn host(&self) -> &Host {
        &self.host
    }

    /// Get the port of the sip uri.
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Get a reference to the `UriParameters` of the sip uri.
    pub fn parameters(&self) -> &UriParameters {
        &self.parameters
    }

    /// Get a reference to the `UriHeaders` of the sip uri.
    pub fn headers(&self) -> &UriHeaders {
        &self.headers
    }
}

impl std::fmt::Display for SipUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}{}{}{}{}{}{}{}{}",
            self.scheme,
            if let Some(userinfo) = &self.userinfo {
                format!("{}", userinfo)
            } else {
                "".to_owned()
            },
            if self.userinfo.is_some() { "@" } else { "" },
            self.host,
            if self.port.is_some() { ":" } else { "" },
            self.port.map(|p| p.to_string()).unwrap_or_default(),
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters,
            if self.headers.is_empty() { "" } else { "?" },
            self.headers
        )
    }
}
