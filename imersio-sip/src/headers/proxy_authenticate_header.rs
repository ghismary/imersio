//! SIP Proxy-Authenticate header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::Challenge;

/// Representation of a Proxy-Authenticate header.
///
/// A Proxy-Authenticate header field value contains an authentication challenge.
///
/// [[RFC3261, Section 20.27](https://datatracker.ietf.org/doc/html/rfc3261#section-20.27)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct ProxyAuthenticateHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    challenge: Challenge,
}

impl ProxyAuthenticateHeader {
    pub(crate) fn new(header: GenericHeader, challenge: Challenge) -> Self {
        Self { header, challenge }
    }

    /// Get a reference to the challenge of the Proxy-Authenticate header.
    pub fn challenge(&self) -> &Challenge {
        &self.challenge
    }
}

impl HeaderAccessor for ProxyAuthenticateHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Proxy-Authenticate")
    }
    fn normalized_value(&self) -> String {
        self.challenge.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Algorithm, DomainUri, Header, ProxyAuthenticateHeader, Uri,
    };
    use claims::assert_ok;

    valid_header!(
        ProxyAuthenticate,
        ProxyAuthenticateHeader,
        "Proxy-Authenticate"
    );
    header_equality!(ProxyAuthenticate, "Proxy-Authenticate");
    header_inequality!(ProxyAuthenticate, "Proxy-Authenticate");

    #[test]
    fn test_valid_proxy_authenticate_header() {
        valid_header(
            r#"Proxy-Authenticate: Digest realm="atlanta.com", domain="sip:ss1.carrier.com", qop="auth", nonce="f84f1cec41e6cbe5aea9c8e88d359", opaque="", stale=FALSE, algorithm=MD5"#,
            |header| {
                let challenge = header.challenge();
                assert_eq!(challenge.scheme(), "Digest");
                assert_eq!(challenge.parameters().len(), 7);
                assert!(challenge.is_digest());
                assert!(challenge.has_realm());
                assert_eq!(challenge.realm(), Some("atlanta.com"));
                assert!(challenge.has_domain());
                assert_eq!(
                    challenge.domain(),
                    Some(
                        &vec![DomainUri::Uri(
                            Uri::try_from("sip:ss1.carrier.com").unwrap()
                        )]
                        .into()
                    )
                );
                // qop
                assert!(challenge.has_nonce());
                assert_eq!(challenge.nonce(), Some("f84f1cec41e6cbe5aea9c8e88d359"));
                assert!(challenge.has_opaque());
                assert_eq!(challenge.opaque(), Some(""));
                assert!(challenge.has_stale());
                assert_eq!(challenge.stale(), Some(&false.into()));
                assert!(challenge.has_algorithm());
                assert_eq!(challenge.algorithm(), Some(&Algorithm::Md5));
            },
        );
    }

    #[test]
    fn test_invalid_proxy_authenticate_header_empty() {
        invalid_header("Proxy-Authenticate:");
    }

    #[test]
    fn test_invalid_proxy_authenticate_header_empty_with_space_characters() {
        invalid_header("Proxy-Authenticate:         ");
    }

    #[test]
    fn test_invalid_proxy_authenticate_header_with_missing_digest_scheme() {
        invalid_header(r#"Proxy-Authenticate: realm="atlanta.com""#);
    }

    #[test]
    fn test_invalid_proxy_authenticate_header_with_missing_quotes_for_qop_param() {
        invalid_header("Proxy-Authenticate: Digest qop=auth");
    }

    #[test]
    fn test_proxy_authenticate_header_equality_with_space_characters_differences() {
        header_equality(
            r#"Proxy-Authenticate: Digest qop="auth,auth-int""#,
            r#"Proxy-Authenticate: Digest  qop="auth,auth-int""#,
        );
    }

    #[test]
    fn test_proxy_authenticate_header_equality_with_different_parameters_order() {
        header_equality(
            r#"Proxy-Authenticate: Digest realm="atlanta.com", nextnonce="47364c23432d2e131a5fb210812c""#,
            r#"Proxy-Authenticate: Digest nextnonce="47364c23432d2e131a5fb210812c", realm="atlanta.com""#,
        );
    }

    #[test]
    fn test_proxy_authenticate_header_equality_with_different_qop_options_order() {
        header_equality(
            r#"Proxy-Authenticate: Digest qop="auth,auth-int""#,
            r#"Proxy-Authenticate: Digest qop="auth-int,auth""#,
        );
    }

    #[test]
    fn test_proxy_authenticate_header_equality_with_different_cases_1() {
        header_equality(
            "Proxy-Authenticate: Digest stale=true",
            "proxy-authenticate: digest  STALE=True",
        );
    }

    #[test]
    fn test_proxy_authenticate_header_equality_with_different_cases_2() {
        header_equality(
            "Proxy-Authenticate: CustomScheme algorithm=MD5-Sess",
            "Proxy-Authenticate: customscheme  Algorithm=Md5-sess",
        );
    }

    #[test]
    fn test_authorization_header_inequality_with_different_parameter_values() {
        header_inequality(
            r#"Proxy-Authenticate: Digest qop="auth""#,
            r#"Proxy-Authenticate: Digest qop="auth-int""#,
        );
    }

    #[test]
    fn test_proxy_authenticate_header_inequality_with_different_parameters() {
        header_inequality(
            r#"Proxy-Authenticate: Digest qop="auth""#,
            r#"Proxy-Authenticate: Digest nextnonce="47364c23432d2e131a5fb210812c""#,
        );
    }

    #[test]
    fn test_proxy_authenticate_header_inequality_with_different_schemes() {
        header_inequality(
            "Proxy-Authenticate: Digest algorithm=MD5",
            "Proxy-Authenticate: CustomScheme algorithm=MD5",
        );
    }

    #[test]
    fn test_proxy_authenticate_header_to_string() {
        let header = Header::try_from(
            r#"ProxY-AuthenticatE  :    Digest   realm="atlanta.com", domain = "sip:ss1.carrier.com", qop="auth", nonce=  "f84f1cec41e6cbe5aea9c8e88d359"  , opaque="", stale  =FALSE, algorithm=MD5"#,
        );
        if let Header::ProxyAuthenticate(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"ProxY-AuthenticatE  :    Digest   realm="atlanta.com", domain = "sip:ss1.carrier.com", qop="auth", nonce=  "f84f1cec41e6cbe5aea9c8e88d359"  , opaque="", stale  =FALSE, algorithm=MD5"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Proxy-Authenticate: Digest realm="atlanta.com", domain="sip:ss1.carrier.com", qop="auth", nonce="f84f1cec41e6cbe5aea9c8e88d359", opaque="", stale=FALSE, algorithm=MD5"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Proxy-Authenticate: Digest realm="atlanta.com", domain="sip:ss1.carrier.com", qop="auth", nonce="f84f1cec41e6cbe5aea9c8e88d359", opaque="", stale=FALSE, algorithm=MD5"#
            );
        }
    }
}
