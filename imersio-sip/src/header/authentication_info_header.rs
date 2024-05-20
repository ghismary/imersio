use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct AuthenticationInfoHeader(Vec<AInfo>);

impl AuthenticationInfoHeader {
    pub(crate) fn new(infos: Vec<AInfo>) -> Self {
        AuthenticationInfoHeader(infos)
    }

    /// Get the number of `AInfo` in the Authentication-Info header.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Tells whether the Authentication-Info header contains a `nextnonce` value.
    pub fn has_next_nonce(&self) -> bool {
        self.0.iter().any(|ai| matches!(ai, AInfo::NextNonce(_)))
    }

    /// Tells whether the Authentication-Info header contains a `qop` value.
    pub fn has_message_qop(&self) -> bool {
        self.0.iter().any(|ai| matches!(ai, AInfo::MessageQop(_)))
    }

    /// Tells whether the Authentication-Info header contains a `rspauth` value.
    pub fn has_response_auth(&self) -> bool {
        self.0.iter().any(|ai| matches!(ai, AInfo::ResponseAuth(_)))
    }

    /// Tells whether the Authentication-Info header contains a `cnonce` value.
    pub fn has_cnonce(&self) -> bool {
        self.0.iter().any(|ai| matches!(ai, AInfo::CNonce(_)))
    }

    /// Tells whether the Authentication-Info header contains a `nc` value.
    pub fn has_nonce_count(&self) -> bool {
        self.0.iter().any(|ai| matches!(ai, AInfo::NonceCount(_)))
    }

    /// Get the `nextnonce` value from the Authentication-Info header.
    pub fn next_nonce(&self) -> Option<&str> {
        self.0
            .iter()
            .find(|ai| matches!(ai, AInfo::NextNonce(_)))
            .map(|ai| {
                if let AInfo::NextNonce(value) = ai {
                    value
                } else {
                    ""
                }
            })
    }

    /// Get the `qop` value from the Authentication-Info header.
    pub fn message_qop(&self) -> Option<&str> {
        self.0
            .iter()
            .find(|ai| matches!(ai, AInfo::MessageQop(_)))
            .map(|ai| {
                if let AInfo::MessageQop(value) = ai {
                    value
                } else {
                    ""
                }
            })
    }

    /// Get the `rspauth` value from the Authentication-Info header.
    pub fn response_auth(&self) -> Option<&str> {
        self.0
            .iter()
            .find(|ai| matches!(ai, AInfo::ResponseAuth(_)))
            .map(|ai| {
                if let AInfo::ResponseAuth(value) = ai {
                    value
                } else {
                    ""
                }
            })
    }

    /// Get the `cnonce` value from the Authentication-Info header.
    pub fn cnonce(&self) -> Option<&str> {
        self.0
            .iter()
            .find(|ai| matches!(ai, AInfo::CNonce(_)))
            .map(|ai| {
                if let AInfo::CNonce(value) = ai {
                    value
                } else {
                    ""
                }
            })
    }

    /// Get the `nc` value from the Authentication-Info header.
    pub fn nonce_count(&self) -> Option<&str> {
        self.0
            .iter()
            .find(|ai| matches!(ai, AInfo::NonceCount(_)))
            .map(|ai| {
                if let AInfo::NonceCount(value) = ai {
                    value
                } else {
                    ""
                }
            })
    }
}

impl std::fmt::Display for AuthenticationInfoHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Authentication-Info: {}",
            self.0
                .iter()
                .map(|info| info.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl PartialEq for AuthenticationInfoHeader {
    fn eq(&self, other: &Self) -> bool {
        let self_ainfos: HashSet<_> = self.0.iter().collect();
        let other_ainfos: HashSet<_> = other.0.iter().collect();
        self_ainfos == other_ainfos
    }
}

impl PartialEq<&AuthenticationInfoHeader> for AuthenticationInfoHeader {
    fn eq(&self, other: &&AuthenticationInfoHeader) -> bool {
        self == *other
    }
}

impl PartialEq<AuthenticationInfoHeader> for &AuthenticationInfoHeader {
    fn eq(&self, other: &AuthenticationInfoHeader) -> bool {
        *self == other
    }
}

impl Eq for AuthenticationInfoHeader {}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AInfo {
    NextNonce(String),
    MessageQop(String),
    ResponseAuth(String),
    CNonce(String),
    NonceCount(String),
}

impl std::fmt::Display for AInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (key, value) = match self {
            Self::NextNonce(value) => ("nextnonce", format!("\"{value}\"")),
            Self::MessageQop(value) => ("qop", value.clone()),
            Self::ResponseAuth(value) => ("rspauth", format!("\"{value}\"")),
            Self::CNonce(value) => ("cnonce", format!("\"{value}\"")),
            Self::NonceCount(value) => ("nc", value.clone()),
        };
        write!(f, "{}={}", key, value)
    }
}

#[cfg(test)]
mod tests {
    use crate::Header;
    use std::str::FromStr;

    #[test]
    fn test_valid_authentication_info_header() {
        let header =
            Header::from_str(r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#);
        assert!(header.is_ok());
        if let Header::AuthenticationInfo(header) = header.unwrap() {
            assert_eq!(header.count(), 1);
            assert!(header.has_next_nonce());
            assert_eq!(header.next_nonce(), Some("47364c23432d2e131a5fb210812c"));
            assert!(!header.has_message_qop());
            assert!(!header.has_cnonce());
            assert!(!header.has_nonce_count());
            assert!(!header.has_response_auth());
        } else {
            panic!("Not an Authentication-Info header");
        }

        let header = Header::from_str("Authentication-Info: qop=auth");
        assert!(header.is_ok());
        if let Header::AuthenticationInfo(header) = header.unwrap() {
            assert_eq!(header.count(), 1);
            assert!(!header.has_next_nonce());
            assert!(header.has_message_qop());
            assert_eq!(header.message_qop(), Some("auth"));
            assert!(!header.has_cnonce());
            assert!(!header.has_nonce_count());
            assert!(!header.has_response_auth());
        } else {
            panic!("Not an Authentication-Info header");
        }
    }

    #[test]
    fn test_invalid_authentication_info_header() {
        // Test empty Authentication-Info header
        let header = Header::from_str("Authentication-Info:");
        assert!(header.is_err());

        // Test empty Authentication-Info header with space characters
        let header = Header::from_str("Authentication-Info:         ");
        assert!(header.is_err());
    }

    #[test]
    fn test_authentication_info_header_equality() {
        let first_header = Header::from_str("Authentication-Info: qop=auth");
        let second_header = Header::from_str("Authentication-Info: qop=auth");
        if let (
            Header::AuthenticationInfo(first_header),
            Header::AuthenticationInfo(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Authentication-Info header");
        }

        let first_header =
            Header::from_str(r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#);
        let second_header =
            Header::from_str(r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#);
        if let (
            Header::AuthenticationInfo(first_header),
            Header::AuthenticationInfo(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Authentication-Info header");
        }
    }

    #[test]
    fn test_authentication_info_header_inequality() {
        let first_header = Header::from_str("Authentication-Info: qop=auth");
        let second_header = Header::from_str("Authentication-Info: qop=auth-int");
        if let (
            Header::AuthenticationInfo(first_header),
            Header::AuthenticationInfo(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Authentication-Info header");
        }

        let first_header = Header::from_str("Authentication-Info: qop=auth");
        let second_header =
            Header::from_str(r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#);
        if let (
            Header::AuthenticationInfo(first_header),
            Header::AuthenticationInfo(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Authentication-Info header");
        }
    }
}
