//! SIP Authentication-Info header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::header::{GenericHeader, HeaderAccessor};
use crate::{AuthenticationInfo, AuthenticationInfos, MessageQop};

/// Representation of an Authentication-Info header.
///
/// The Authentication-Info header field provides for mutual authentication
/// with HTTP Digest.
///
/// [[RFC3261, Section 20.6](https://datatracker.ietf.org/doc/html/rfc3261#section-20.6)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct AuthenticationInfoHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    infos: AuthenticationInfos,
}

impl AuthenticationInfoHeader {
    pub(crate) fn new(header: GenericHeader, infos: Vec<AuthenticationInfo>) -> Self {
        Self {
            header,
            infos: infos.into(),
        }
    }

    /// Get a reference to the `AInfos` from the Authentication-Info header.
    pub fn infos(&self) -> &AuthenticationInfos {
        &self.infos
    }

    /// Tell whether the Authentication-Info header contains a `qop` value.
    pub fn has_qop(&self) -> bool {
        self.infos
            .iter()
            .any(|ai| matches!(ai, AuthenticationInfo::Qop(_)))
    }

    /// Get the `qop` value from the Authentication-Info header.
    pub fn qop(&self) -> Option<&MessageQop> {
        self.infos
            .iter()
            .find(|ai| matches!(ai, AuthenticationInfo::Qop(_)))
            .and_then(|ai| {
                if let AuthenticationInfo::Qop(value) = ai {
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
                    self.infos.iter().any(|ai| matches!(ai, AuthenticationInfo::$enum_name(_)))
                }

                /// Get the `$token` value from the Authentication-Info header.
                pub fn $token(&self) -> Option<&str> {
                    self.infos
                        .iter()
                        .find(|ai| matches!(ai, AuthenticationInfo::$enum_name(_)))
                        .map(|ai| {
                            if let AuthenticationInfo::$enum_name(value) = ai {
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

#[cfg(test)]
mod tests {
    use crate::{
        header::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        AuthenticationInfoHeader, Header, MessageQop,
    };
    use claims::assert_ok;

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
                assert!(!header.has_qop());
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
            assert!(header.has_qop());
            assert_eq!(header.qop(), Some(&MessageQop::Auth));
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
        let header = Header::try_from(
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
