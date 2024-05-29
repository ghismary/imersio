use partial_eq_refs::PartialEqRefs;

use crate::{HostPort, UriHeaders, UriParameters, UriScheme, UserInfo};

/// Representation of a SIP URI.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, PartialEqRefs)]
pub struct SipUri {
    scheme: UriScheme,
    userinfo: Option<UserInfo>,
    hostport: HostPort,
    parameters: UriParameters,
    headers: UriHeaders,
}

impl SipUri {
    pub(crate) fn new(
        scheme: UriScheme,
        userinfo: Option<UserInfo>,
        hostport: HostPort,
        parameters: UriParameters,
        headers: UriHeaders,
    ) -> Self {
        Self {
            scheme,
            userinfo,
            hostport,
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

    /// Get a reference to the `HostPort` of the sip uri.
    pub fn hostport(&self) -> &HostPort {
        &self.hostport
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
            "{}:{}{}{}{}{}{}{}",
            self.scheme,
            if let Some(userinfo) = &self.userinfo {
                format!("{}", userinfo)
            } else {
                "".to_owned()
            },
            if self.userinfo.is_some() { "@" } else { "" },
            self.hostport,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters,
            if self.headers.is_empty() { "" } else { "?" },
            self.headers
        )
    }
}
