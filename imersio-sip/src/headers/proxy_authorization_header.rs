//! SIP Proxy-Authorization header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::Credentials;

/// Representation of a Proxy-Authorization header.
///
/// The Proxy-Authorization header field allows the client to identify itself (or its user) to a
/// proxy that requires authentication. A Proxy-Authorization field value consists of credentials
/// containing the authentication information of the user agent for the proxy and/or realm of the
/// resource being requested.
///
/// [[RFC3261, Section 20.28](https://datatracker.ietf.org/doc/html/rfc3261#section-20.28)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct ProxyAuthorizationHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    credentials: Credentials,
}

impl ProxyAuthorizationHeader {
    pub(crate) fn new(header: GenericHeader, credentials: Credentials) -> Self {
        Self {
            header,
            credentials,
        }
    }

    /// Get a reference to the `Credentials` of the Proxy-Authorization header.
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}

impl HeaderAccessor for ProxyAuthorizationHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Proxy-Authorization")
    }
    fn normalized_value(&self) -> String {
        self.credentials.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::credentials::parser::credentials;
    use crate::headers::GenericHeader;
    use crate::parser::{hcolon, ParserResult};
    use crate::{Header, ProxyAuthorizationHeader};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        sequence::tuple,
    };

    pub(crate) fn proxy_authorization(input: &str) -> ParserResult<&str, Header> {
        context(
            "Proxy-Authorization header",
            map(
                tuple((
                    tag_no_case("Proxy-Authorization"),
                    hcolon,
                    cut(consumed(credentials)),
                )),
                |(name, separator, (value, credentials))| {
                    Header::ProxyAuthorization(ProxyAuthorizationHeader::new(
                        GenericHeader::new(name, separator, value),
                        credentials,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::{algorithm::Algorithm, message_qop::MessageQop},
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        AuthParameter, Header, ProxyAuthorizationHeader, Uri,
    };
    use claims::assert_ok;

    valid_header!(
        ProxyAuthorization,
        ProxyAuthorizationHeader,
        "Proxy-Authorization"
    );
    header_equality!(ProxyAuthorization, "Proxy-Authorization");
    header_inequality!(ProxyAuthorization, "Proxy-Authorization");

    #[test]
    fn test_valid_proxy_authorization_header_1() {
        valid_header(
            r#"Proxy-Authorization: Digest username="Alice", realm="atlanta.com", nonce="c60f3082ee1212b402a21831ae", response="245f23415f11432b3434341c022a5432""#,
            |header| {
                let credentials = header.credentials();
                assert_eq!(credentials.scheme(), "Digest");
                assert_eq!(credentials.parameters().len(), 4);
                assert!(credentials.is_digest());
                assert!(credentials.has_username());
                assert_eq!(credentials.username(), Some("Alice"));
                assert!(credentials.has_realm());
                assert_eq!(credentials.realm(), Some("atlanta.com"));
                assert!(credentials.has_nonce());
                assert_eq!(credentials.nonce(), Some("c60f3082ee1212b402a21831ae"));
                assert!(credentials.has_dresponse());
                assert_eq!(
                    credentials.dresponse(),
                    Some("245f23415f11432b3434341c022a5432")
                );
                assert!(!credentials.has_algorithm());
                assert_eq!(credentials.algorithm(), None);
                assert!(!credentials.has_digest_uri());
                assert_eq!(credentials.digest_uri(), None);
            },
        );
    }

    #[test]
    fn test_valid_proxy_authorization_header_2() {
        valid_header(
            r#"Proxy-Authorization: Digest username="bob", realm="biloxi.com", nonce="dcd98b7102dd2f0e8b11d0f600bfb0c093", uri="sip:bob@biloxi.com", qop=auth, nc=00000001, cnonce="0a4f113b", response="6629fae49393a05397450978507c4ef1", opaque="5ccc069c403ebaf9f0171e9517f40e41""#,
            |header| {
                let credentials = header.credentials();
                assert_eq!(credentials.scheme(), "Digest");
                assert_eq!(credentials.parameters().len(), 9);
                assert!(credentials.has_username());
                assert_eq!(credentials.username(), Some("bob"));
                assert!(credentials.has_realm());
                assert_eq!(credentials.realm(), Some("biloxi.com"));
                assert!(credentials.has_nonce());
                assert_eq!(
                    credentials.nonce(),
                    Some("dcd98b7102dd2f0e8b11d0f600bfb0c093")
                );
                assert!(credentials.has_digest_uri());
                assert_eq!(
                    credentials.digest_uri().unwrap(),
                    Uri::try_from("sip:bob@biloxi.com").unwrap()
                );
                assert!(credentials.has_qop());
                assert_eq!(credentials.qop(), Some(&MessageQop::Auth));
                assert!(credentials.has_nonce_count());
                assert_eq!(credentials.nonce_count(), Some("00000001"));
                assert!(credentials.has_cnonce());
                assert_eq!(credentials.cnonce(), Some("0a4f113b"));
                assert!(credentials.has_dresponse());
                assert_eq!(
                    credentials.dresponse(),
                    Some("6629fae49393a05397450978507c4ef1")
                );
                assert!(credentials.has_opaque());
                assert_eq!(
                    credentials.opaque(),
                    Some("5ccc069c403ebaf9f0171e9517f40e41")
                );
                assert!(!credentials.has_algorithm());
                assert_eq!(credentials.algorithm(), None);
            },
        );
    }

    #[test]
    fn test_valid_proxy_authorization_header_with_algorithm() {
        valid_header("Proxy-Authorization: Digest algorithm=MD5", |header| {
            let credentials = header.credentials();
            assert_eq!(credentials.scheme(), "Digest");
            assert!(credentials.has_algorithm());
            assert_eq!(
                credentials.parameters().first().unwrap(),
                &AuthParameter::Algorithm(Algorithm::Md5)
            );
            assert!(credentials.contains("algorithm"));
            assert_eq!(
                credentials.get("algorithm"),
                Some(&AuthParameter::Algorithm(Algorithm::Md5))
            );
        });
    }

    #[test]
    fn test_valid_proxy_authorization_header_with_custom_scheme() {
        valid_header(
            "Proxy-Authorization: CustomScheme customparam=value",
            |header| {
                let credentials = header.credentials();
                assert_eq!(credentials.scheme(), "CustomScheme");
                assert!(!credentials.has_algorithm());
                assert_eq!(credentials.algorithm(), None);
                assert!(credentials.contains("customparam"));
                assert_eq!(credentials.get("customparam").unwrap().value(), "value");
                assert!(!credentials.contains("customparam2"));
                assert_eq!(credentials.get("customparam2"), None);
            },
        );
    }

    #[test]
    fn test_invalid_proxy_authorization_header_empty() {
        invalid_header("Proxy-Authorization:");
    }

    #[test]
    fn test_invalid_proxy_authorization_header_empty_with_space_characters() {
        invalid_header("Proxy-Authorization:         ");
    }

    #[test]
    fn test_invalid_proxy_authorization_header_with_response_that_is_too_long() {
        invalid_header(
            r#"Proxy-Authorization: Digest response="6629fae49393a05397450978507c4ef12""#,
        );
    }

    #[test]
    fn test_invalid_proxy_authorization_header_with_response_that_is_too_short() {
        invalid_header(r#"Proxy-Authorization: Digest response="6629fae49393a0""#);
    }

    #[test]
    fn test_invalid_proxy_authorization_header_with_missing_digest_scheme() {
        invalid_header("Proxy-Authorization: qop=auth");
    }

    #[test]
    fn test_proxy_authorization_header_equality_with_space_characters_differences() {
        header_equality(
            "Proxy-Authorization: Digest qop=auth",
            "Proxy-Authorization: Digest  qop=auth",
        );
    }

    #[test]
    fn test_proxy_authorization_header_equality_with_different_parameters_order() {
        header_equality(
            r#"Proxy-Authorization: Digest username="Alice", nextnonce="47364c23432d2e131a5fb210812c""#,
            r#"Proxy-Authorization: Digest nextnonce="47364c23432d2e131a5fb210812c", username="Alice""#,
        );
    }

    #[test]
    fn test_proxy_authorization_header_equality_with_different_cases_1() {
        header_equality(
            "Proxy-Authorization: Digest qop=auth",
            "Proxy-authorization: digest  QOP=Auth",
        );
    }

    #[test]
    fn test_proxy_authorization_header_equality_with_different_cases_2() {
        header_equality(
            "Proxy-Authorization: CustomScheme algorithm=MD5-Sess",
            "proxy-authorization: customscheme  Algorithm=Md5-sess",
        );
    }

    #[test]
    fn test_proxy_authorization_header_inequality_with_different_parameter_values() {
        header_inequality(
            "Proxy-Authorization: Digest qop=auth",
            "Proxy-Authorization: Digest qop=auth-int",
        );
    }

    #[test]
    fn test_proxy_authorization_header_inequality_with_different_parameters() {
        header_inequality(
            "Proxy-Authorization: Digest qop=auth",
            r#"Proxy-Authorization: Digest nextnonce="47364c23432d2e131a5fb210812c""#,
        );
    }

    #[test]
    fn test_proxy_authorization_header_inequality_with_different_schemes() {
        header_inequality(
            "Proxy-Authorization: Digest algorithm=MD5",
            "Proxy-Authorization: CustomScheme algorithm=MD5",
        );
    }

    #[test]
    fn test_proxy_authorization_header_to_string_with_username_and_qop_parameter() {
        let header =
            Header::try_from(r#"proxY-authorization:  diGest username ="Alice" ,   qop= AUTH"#);
        if let Header::Authorization(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"proxY-authorization:  diGest username ="Alice" ,   qop= AUTH"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Proxy-Authorization: Digest username="Alice", qop=auth"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Proxy-Authorization: Digest username="Alice", qop=auth"#
            );
        }
    }

    #[test]
    fn test_proxy_authorization_header_to_string_with_extension_parameter() {
        let header = Header::try_from(
            r#"proXy-authorization:  diGest username ="Alice" ,   nextnonce= "47364c23432d2e131a5fb210812c""#,
        );
        if let Header::Authorization(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"proXy-authorization:  diGest username ="Alice" ,   nextnonce= "47364c23432d2e131a5fb210812c""#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Proxy-Authorization: Digest username="Alice", nextnonce="47364c23432d2e131a5fb210812c""#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Proxy-Authorization: Digest username="Alice", nextnonce="47364c23432d2e131a5fb210812c""#
            );
        }
    }
}
