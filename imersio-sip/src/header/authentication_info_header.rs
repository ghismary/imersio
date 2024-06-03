use std::hash::Hash;

use partial_eq_refs::PartialEqRefs;

use crate::{
    common::{
        header_value_collection::HeaderValueCollection, message_qop::MessageQop,
        wrapped_string::WrappedString,
    },
    HeaderAccessor,
};

use super::generic_header::GenericHeader;

/// Representation of an Authentication-Info header.
///
/// The Authentication-Info header field provides for mutual authentication
/// with HTTP Digest.
///
/// [[RFC3261, Section 20.6](https://datatracker.ietf.org/doc/html/rfc3261#section-20.6)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct AuthenticationInfoHeader {
    header: GenericHeader,
    infos: AInfos,
}

impl AuthenticationInfoHeader {
    pub(crate) fn new(header: GenericHeader, infos: Vec<AInfo>) -> Self {
        Self {
            header,
            infos: infos.into(),
        }
    }

    /// Get a reference to the `AInfos` from the Authentication-Info header.
    pub fn infos(&self) -> &AInfos {
        &self.infos
    }

    /// Tell whether the Authentication-Info header contains a `qop` value.
    pub fn has_message_qop(&self) -> bool {
        self.infos
            .iter()
            .any(|ai| matches!(ai, AInfo::MessageQop(_)))
    }

    /// Get the `qop` value from the Authentication-Info header.
    pub fn message_qop(&self) -> Option<&MessageQop> {
        self.infos
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

impl HeaderAccessor for AuthenticationInfoHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Authentication-Info")
    }
    fn normalized_value(&self) -> String {
        self.infos.to_string()
    }
}

impl std::fmt::Display for AuthenticationInfoHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for AuthenticationInfoHeader {
    fn eq(&self, other: &Self) -> bool {
        self.infos == other.infos
    }
}

macro_rules! authentication_info_header {
    (
        $(
            ($token:ident, $has_token:ident, $enum_name:ident),
        )+
    ) => {
        impl AuthenticationInfoHeader {
            $(
                /// Tell whether the Authentication-Info header contains a `$token` value.
                pub fn $has_token(&self) -> bool {
                    self.infos.iter().any(|ai| matches!(ai, AInfo::$enum_name(_)))
                }

                /// Get the `$token` value from the Authentication-Info header.
                pub fn $token(&self) -> Option<&str> {
                    self.infos
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
    (next_nonce, has_next_nonce, NextNonce),
    (response_auth, has_response_auth, ResponseAuth),
    (cnonce, has_cnonce, CNonce),
    (nonce_count, has_nonce_count, NonceCount),
}

/// Representation of the list of authentication infos from an
/// `AuthenticationInfoHeader`.
///
/// This is usable as an iterator.
pub type AInfos = HeaderValueCollection<AInfo>;

/// Representation of an info from an `AuthenticationInfoHeader`.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
#[non_exhaustive]
pub enum AInfo {
    /// A `nextnonce` authentication info.
    NextNonce(WrappedString),
    /// A `qop` authentication info.
    MessageQop(MessageQop),
    /// A `rspauth` authentication info.
    ResponseAuth(WrappedString),
    /// A `cnonce` authentication info.
    CNonce(WrappedString),
    /// A `nonce` authentication info.
    NonceCount(WrappedString),
}

impl AInfo {
    /// Get the key of the authentication info.
    pub fn key(&self) -> &str {
        match self {
            Self::NextNonce(_) => "nextnonce",
            Self::MessageQop(_) => "qop",
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
            Self::MessageQop(value) => value.value(),
        }
    }
}

impl std::fmt::Display for AInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (key, value) = match self {
            Self::NextNonce(value) => ("nextnonce", value.to_string()),
            Self::MessageQop(value) => ("qop", value.to_string()),
            Self::ResponseAuth(value) => ("rspauth", value.to_string()),
            Self::CNonce(value) => ("cnonce", value.to_string()),
            Self::NonceCount(value) => ("nc", value.to_string()),
        };
        write!(f, "{}={}", key, value)
    }
}

impl PartialEq<AInfo> for AInfo {
    fn eq(&self, other: &AInfo) -> bool {
        match (self, other) {
            (Self::NextNonce(a), Self::NextNonce(b))
            | (Self::ResponseAuth(a), Self::ResponseAuth(b))
            | (Self::CNonce(a), Self::CNonce(b))
            | (Self::NonceCount(a), Self::NonceCount(b)) => a == b,
            (Self::MessageQop(a), Self::MessageQop(b)) => a == b,
            _ => false,
        }
    }
}

impl Hash for AInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        match self {
            Self::NextNonce(value)
            | Self::ResponseAuth(value)
            | Self::CNonce(value)
            | Self::NonceCount(value) => value.hash(state),
            Self::MessageQop(value) => value.value().to_ascii_lowercase().hash(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthenticationInfoHeader;
    use crate::{
        common::message_qop::MessageQop,
        header::tests::{header_equality, header_inequality, invalid_header, valid_header},
        Header, HeaderAccessor,
    };
    use claims::assert_ok;
    use std::str::FromStr;

    valid_header!(
        AuthenticationInfo,
        AuthenticationInfoHeader,
        "Authentication-Info"
    );
    header_equality!(AuthenticationInfo, "Authentication-Info");
    header_inequality!(AuthenticationInfo, "Authentication-Info");

    #[test]
    fn test_valid_authentication_info_header_with_nextnonce() {
        valid_header(
            r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#,
            |header| {
                assert_eq!(header.infos().len(), 1);
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
            assert_eq!(header.infos().len(), 1);
            assert!(!header.has_next_nonce());
            assert!(header.has_message_qop());
            assert_eq!(header.message_qop(), Some(&MessageQop::Auth));
            assert!(!header.has_cnonce());
            assert!(!header.has_nonce_count());
            assert!(!header.has_response_auth());
        });
    }

    #[test]
    fn test_invalid_authentication_info_header_empty() {
        invalid_header("Authentication-Info:");
    }

    #[test]
    fn test_invalid_authentication_info_header_empty_with_space_characters() {
        invalid_header("Authentication-Info:         ");
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

    #[test]
    fn test_authentication_info_header_to_string() {
        let header = Header::from_str(
            r#"authentication-info:   nextNonce =   "47364c23432d2e131a5fb210812c""#,
        );
        if let Header::AuthenticationInfo(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"authentication-info:   nextNonce =   "47364c23432d2e131a5fb210812c""#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Authentication-Info: nextnonce="47364c23432d2e131a5fb210812c""#
            );
        }
    }
}
