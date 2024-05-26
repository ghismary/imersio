use std::{collections::HashSet, hash::Hash};

use crate::common::MessageQop;

#[derive(Clone, Debug, Eq)]
pub struct AuthenticationInfoHeader(Vec<AInfo>);

impl AuthenticationInfoHeader {
    pub(crate) fn new(infos: Vec<AInfo>) -> Self {
        AuthenticationInfoHeader(infos)
    }

    /// Get the number of `AInfo` in the Authentication-Info header.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Tells whether the Authentication-Info header contains a `qop` value.
    pub fn has_message_qop(&self) -> bool {
        self.0.iter().any(|ai| matches!(ai, AInfo::MessageQop(_)))
    }

    /// Get the `qop` value from the Authentication-Info header.
    pub fn message_qop(&self) -> Option<&MessageQop> {
        self.0
            .iter()
            .find(|ai| matches!(ai, AInfo::MessageQop(_)))
            .and_then(|ai| {
                if let AInfo::MessageQop(value) = ai {
                    Some(value)
                } else {
                    None
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

macro_rules! authentication_info_header {
    (
        $(
            ($token:ident, $has_token:ident, $enum_name:ident);
        )+
    ) => {
        impl AuthenticationInfoHeader {
            $(
                /// Tells whether the Authentication-Info header contains a `$token` value.
                pub fn $has_token(&self) -> bool {
                    self.0.iter().any(|ai| matches!(ai, AInfo::$enum_name(_)))
                }

                /// Get the `$token` value from the Authentication-Info header.
                pub fn $token(&self) -> Option<&str> {
                    self.0
                        .iter()
                        .find(|ai| matches!(ai, AInfo::$enum_name(_)))
                        .map(|ai| {
                            if let AInfo::$enum_name(value) = ai {
                                value
                            } else {
                                ""
                            }
                        })
                }
            )+
        }
    }
}

authentication_info_header! {
    (next_nonce, has_next_nonce, NextNonce);
    (response_auth, has_response_auth, ResponseAuth);
    (cnonce, has_cnonce, CNonce);
    (nonce_count, has_nonce_count, NonceCount);
}

#[derive(Clone, Debug, Eq)]
#[non_exhaustive]
pub enum AInfo {
    NextNonce(String),
    MessageQop(MessageQop),
    ResponseAuth(String),
    CNonce(String),
    NonceCount(String),
}

impl AInfo {
    pub fn key(&self) -> &str {
        match self {
            Self::NextNonce(_) => "nextnonce",
            Self::MessageQop(_) => "qop",
            Self::ResponseAuth(_) => "rspauth",
            Self::CNonce(_) => "cnonce",
            Self::NonceCount(_) => "nc",
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::NextNonce(value)
            | Self::ResponseAuth(value)
            | Self::CNonce(value)
            | Self::NonceCount(value) => value,
            Self::MessageQop(value) => value.value(),
        }
    }
}

impl std::fmt::Display for AInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (key, value) = match self {
            Self::NextNonce(value) => ("nextnonce", format!("\"{value}\"")),
            Self::MessageQop(value) => ("qop", value.value().to_string()),
            Self::ResponseAuth(value) => ("rspauth", format!("\"{value}\"")),
            Self::CNonce(value) => ("cnonce", format!("\"{value}\"")),
            Self::NonceCount(value) => ("nc", value.clone()),
        };
        write!(f, "{}={}", key, value)
    }
}

impl PartialEq<AInfo> for AInfo {
    fn eq(&self, other: &AInfo) -> bool {
        match (self, other) {
            (Self::NextNonce(a), Self::NextNonce(b))
            | (Self::ResponseAuth(a), Self::ResponseAuth(b))
            | (Self::CNonce(a), Self::CNonce(b)) => a == b,
            (Self::MessageQop(a), Self::MessageQop(b)) => a == b,
            (Self::NonceCount(a), Self::NonceCount(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialEq<&AInfo> for AInfo {
    fn eq(&self, other: &&AInfo) -> bool {
        self == *other
    }
}

impl PartialEq<AInfo> for &AInfo {
    fn eq(&self, other: &AInfo) -> bool {
        *self == other
    }
}

impl Hash for AInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        match self {
            Self::NextNonce(value) | Self::ResponseAuth(value) | Self::CNonce(value) => {
                value.hash(state)
            }
            Self::MessageQop(value) => value.value().to_ascii_lowercase().hash(state),
            Self::NonceCount(value) => value.to_ascii_lowercase().hash(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthenticationInfoHeader;
    use crate::{common::MessageQop, Header};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(AuthenticationInfoHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert!(header.is_ok());
        if let Header::AuthenticationInfo(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not an Authentication-Info header");
        }
    }

    #[test]
    fn test_valid_authentication_info_header_with_nextnonce() {
        valid_header(
            r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#,
            |header| {
                assert_eq!(header.count(), 1);
                assert!(header.has_next_nonce());
                assert_eq!(header.next_nonce(), Some("47364c23432d2e131a5fb210812c"));
                assert!(!header.has_message_qop());
                assert!(!header.has_cnonce());
                assert!(!header.has_nonce_count());
                assert!(!header.has_response_auth());
            },
        );
    }

    #[test]
    fn test_valid_authentication_info_header_with_qop() {
        valid_header("Authentication-Info: qop=auth", |header| {
            assert_eq!(header.count(), 1);
            assert!(!header.has_next_nonce());
            assert!(header.has_message_qop());
            assert_eq!(header.message_qop(), Some(&MessageQop::Auth));
            assert!(!header.has_cnonce());
            assert!(!header.has_nonce_count());
            assert!(!header.has_response_auth());
        });
    }

    fn invalid_header(header: &str) {
        assert!(Header::from_str(header).is_err());
    }

    #[test]
    fn test_invalid_authentication_info_header_empty() {
        invalid_header("Authentication-Info:");
    }

    #[test]
    fn test_invalid_authentication_info_header_empty_with_space_characters() {
        invalid_header("Authentication-Info:         ");
    }

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
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
    fn test_authentication_info_header_equality_same_headers() {
        header_equality(
            r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#,
            r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#,
        );
    }

    #[test]
    fn test_authentication_info_header_equality_with_space_characters_differences() {
        header_equality(
            "Authentication-Info: qop=auth",
            "Authentication-Info:   qop=auth",
        );
    }

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
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

    #[test]
    fn test_authentication_info_header_inequality_different_parameter_values() {
        header_inequality(
            "Authentication-Info: qop=auth",
            "Authentication-Info: qop=auth-int",
        );
    }

    #[test]
    fn test_authentication_info_header_inequality_different_parameters() {
        header_inequality(
            "Authentication-Info: qop=auth",
            r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#,
        );
    }
}
