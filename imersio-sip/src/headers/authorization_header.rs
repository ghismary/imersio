//! SIP Authorization header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::Credentials;

/// Representation of an Authorization header.
///
/// The Authorization header field contains authentication credentials of a UA.
///
/// [[RFC3261, Section 20.7](https://datatracker.ietf.org/doc/html/rfc3261#section-20.7)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display("{}", header)]
pub struct AuthorizationHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    credentials: Credentials,
}

impl AuthorizationHeader {
    pub(crate) fn new(header: GenericHeader, credentials: Credentials) -> Self {
        Self {
            header,
            credentials,
        }
    }

    /// Get a reference to the `Credentials` of the Authorization header.
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}

impl HeaderAccessor for AuthorizationHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Authorization")
    }
    fn normalized_value(&self) -> String {
        self.credentials.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::credentials::parser::credentials;
    use crate::headers::GenericHeader;
    use crate::parser::{hcolon, ParserResult};
    use crate::{AuthorizationHeader, Header};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        sequence::tuple,
    };

    pub(crate) fn authorization(input: &str) -> ParserResult<&str, Header> {
        context(
            "Authorization header",
            map(
                tuple((
                    tag_no_case("Authorization"),
                    hcolon,
                    cut(consumed(credentials)),
                )),
                |(name, separator, (value, credentials))| {
                    Header::Authorization(AuthorizationHeader::new(
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
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Algorithm, AuthParameter, AuthorizationHeader, Header, MessageQop, Uri,
    };
    use claims::assert_ok;

    valid_header!(Authorization, AuthorizationHeader, "Authorization");
    header_equality!(Authorization, "Authorization");
    header_inequality!(Authorization, "Authorization");

    #[test]
    fn test_valid_authorization_header_1() {
        valid_header(
            r#"Authorization: Digest username="Alice", realm="atlanta.com", nonce="84a4cc6f3082121f32b42a2187831a9e", response="7587245234b3434cc3412213e5f113a5""#,
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
                assert_eq!(
                    credentials.nonce(),
                    Some("84a4cc6f3082121f32b42a2187831a9e")
                );
                assert!(credentials.has_dresponse());
                assert_eq!(
                    credentials.dresponse(),
                    Some("7587245234b3434cc3412213e5f113a5")
                );
                assert!(!credentials.has_algorithm());
                assert_eq!(credentials.algorithm(), None);
                assert!(!credentials.has_digest_uri());
                assert_eq!(credentials.digest_uri(), None);
            },
        );
    }

    #[test]
    fn test_valid_authorization_header_2() {
        valid_header(
            r#"Authorization: Digest username="bob", realm="biloxi.com", nonce="dcd98b7102dd2f0e8b11d0f600bfb0c093", uri="sip:bob@biloxi.com", qop=auth, nc=00000001, cnonce="0a4f113b", response="6629fae49393a05397450978507c4ef1", opaque="5ccc069c403ebaf9f0171e9517f40e41""#,
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
    fn test_valid_authorization_header_with_algorithm() {
        valid_header("Authorization: Digest algorithm=MD5", |header| {
            let credentials = header.credentials();
            assert_eq!(credentials.scheme(), "Digest");
            assert!(credentials.has_algorithm());
            assert_eq!(
                credentials.parameters().first().unwrap(),
                AuthParameter::Algorithm(Algorithm::Md5)
            );
            assert!(credentials.contains("algorithm"));
            assert_eq!(
                credentials.get("algorithm"),
                Some(&AuthParameter::Algorithm(Algorithm::Md5))
            );
        });
    }

    #[test]
    fn test_valid_authorization_header_with_custom_scheme() {
        valid_header("Authorization: CustomScheme customparam=value", |header| {
            let credentials = header.credentials();
            assert_eq!(credentials.scheme(), "CustomScheme");
            assert!(!credentials.has_algorithm());
            assert_eq!(credentials.algorithm(), None);
            assert!(credentials.contains("customparam"));
            assert_eq!(credentials.get("customparam").unwrap().value(), "value");
            assert!(!credentials.contains("customparam2"));
            assert_eq!(credentials.get("customparam2"), None);
        });
    }

    #[test]
    fn test_invalid_authorization_header_empty() {
        invalid_header("Authorization:");
    }

    #[test]
    fn test_invalid_authorization_header_empty_with_space_characters() {
        invalid_header("Authorization:         ");
    }

    #[test]
    fn test_invalid_authorization_header_with_response_that_is_too_long() {
        invalid_header(r#"Authorization: Digest response="6629fae49393a05397450978507c4ef12""#);
    }

    #[test]
    fn test_invalid_authorization_header_with_response_that_is_too_short() {
        invalid_header(r#"Authorization: Digest response="6629fae49393a0""#);
    }

    #[test]
    fn test_invalid_authorization_header_with_missing_digest_scheme() {
        invalid_header("Authorization: qop=auth");
    }

    #[test]
    fn test_authorization_header_equality_with_space_characters_differences() {
        header_equality(
            "Authorization: Digest qop=auth",
            "Authorization: Digest  qop=auth",
        );
    }

    #[test]
    fn test_authorization_header_equality_with_different_parameters_order() {
        header_equality(
            r#"Authorization: Digest username="Alice", nextnonce="47364c23432d2e131a5fb210812c""#,
            r#"Authorization: Digest nextnonce="47364c23432d2e131a5fb210812c", username="Alice""#,
        );
    }

    #[test]
    fn test_authorization_header_equality_with_different_cases_1() {
        header_equality(
            "Authorization: Digest qop=auth",
            "authorization: digest  QOP=Auth",
        );
    }

    #[test]
    fn test_authorization_header_equality_with_different_cases_2() {
        header_equality(
            "Authorization: CustomScheme algorithm=MD5-Sess",
            "authorization: customscheme  Algorithm=Md5-sess",
        );
    }

    #[test]
    fn test_authorization_header_inequality_with_different_parameter_values() {
        header_inequality(
            "Authorization: Digest qop=auth",
            "Authorization: Digest qop=auth-int",
        );
    }

    #[test]
    fn test_authorization_header_inequality_with_different_parameters() {
        header_inequality(
            "Authorization: Digest qop=auth",
            r#"Authorization: Digest nextnonce="47364c23432d2e131a5fb210812c""#,
        );
    }

    #[test]
    fn test_authorization_header_inequality_with_different_schemes() {
        header_inequality(
            "Authorization: Digest algorithm=MD5",
            "Authorization: CustomScheme algorithm=MD5",
        );
    }

    #[test]
    fn test_authorization_header_to_string_with_username_and_qop_parameter() {
        let header = Header::try_from(r#"authorization:  diGest username ="Alice" ,   qop= AUTH"#);
        if let Header::Authorization(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"authorization:  diGest username ="Alice" ,   qop= AUTH"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Authorization: Digest username="Alice", qop=auth"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Authorization: Digest username="Alice", qop=auth"#
            );
        }
    }

    #[test]
    fn test_authorization_header_to_string_with_extension_parameter() {
        let header = Header::try_from(
            r#"authorization:  diGest username ="Alice" ,   nextnonce= "47364c23432d2e131a5fb210812c""#,
        );
        if let Header::Authorization(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"authorization:  diGest username ="Alice" ,   nextnonce= "47364c23432d2e131a5fb210812c""#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Authorization: Digest username="Alice", nextnonce="47364c23432d2e131a5fb210812c""#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Authorization: Digest username="Alice", nextnonce="47364c23432d2e131a5fb210812c""#
            );
        }
    }
}
