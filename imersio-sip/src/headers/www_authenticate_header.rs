//! SIP WWW-Authenticate header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::Challenge;

/// Representation of a WWW-Authenticate header.
///
/// A WWW-Authenticate header field value contains an authentication challenge.
///
/// [[RFC3261, Section 20.44](https://datatracker.ietf.org/doc/html/rfc3261#section-20.44)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct WWWAuthenticateHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    challenge: Challenge,
}

impl WWWAuthenticateHeader {
    pub(crate) fn new(header: GenericHeader, challenge: Challenge) -> Self {
        Self { header, challenge }
    }

    /// Get a reference to the challenge of the WWW-Authenticate header.
    pub fn challenge(&self) -> &Challenge {
        &self.challenge
    }
}

impl HeaderAccessor for WWWAuthenticateHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("WWW-Authenticate")
    }
    fn normalized_value(&self) -> String {
        self.challenge.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::challenge::parser::challenge;
    use crate::headers::GenericHeader;
    use crate::parser::{hcolon, ParserResult};
    use crate::{Header, WWWAuthenticateHeader};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        sequence::tuple,
    };

    pub(crate) fn www_authenticate(input: &str) -> ParserResult<&str, Header> {
        context(
            "WWW-Authenticate header",
            map(
                tuple((
                    tag_no_case("WWW-Authenticate"),
                    hcolon,
                    cut(consumed(challenge)),
                )),
                |(name, separator, (value, challenge))| {
                    Header::WWWAuthenticate(WWWAuthenticateHeader::new(
                        GenericHeader::new(name, separator, value),
                        challenge,
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
        Algorithm, DomainUri, Header, MessageQop, Uri, WWWAuthenticateHeader,
    };
    use claims::assert_ok;

    valid_header!(WWWAuthenticate, WWWAuthenticateHeader, "WWW-Authenticate");
    header_equality!(WWWAuthenticate, "WWW-Authenticate");
    header_inequality!(WWWAuthenticate, "WWW-Authenticate");

    #[test]
    fn test_valid_www_authenticate_header() {
        valid_header(
            r#"WWW-Authenticate: Digest realm="atlanta.com", domain="sip:boxesbybob.com", qop="auth", nonce="f84f1cec41e6cbe5aea9c8e88d359", opaque="", stale=FALSE, algorithm=MD5"#,
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
                        &vec![DomainUri::Uri(Uri::try_from("sip:boxesbybob.com").unwrap())].into()
                    )
                );
                assert!(challenge.has_qop());
                assert_eq!(challenge.qop().unwrap().len(), 1);
                assert_eq!(challenge.qop().unwrap().first().unwrap(), MessageQop::Auth);
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
    fn test_invalid_www_authenticate_header_empty() {
        invalid_header("WWW-Authenticate:");
    }

    #[test]
    fn test_invalid_www_authenticate_header_empty_with_space_characters() {
        invalid_header("WWW-Authenticate:         ");
    }

    #[test]
    fn test_invalid_www_authenticate_header_with_missing_digest_scheme() {
        invalid_header(r#"WWW-Authenticate: realm="atlanta.com""#);
    }

    #[test]
    fn test_invalid_www_authenticate_header_with_missing_quotes_for_qop_param() {
        invalid_header("WWW-Authenticate: Digest qop=auth");
    }

    #[test]
    fn test_www_authenticate_header_equality_with_space_characters_differences() {
        header_equality(
            r#"WWW-Authenticate: Digest qop="auth,auth-int""#,
            r#"WWW-Authenticate: Digest  qop="auth,auth-int""#,
        );
    }

    #[test]
    fn test_www_authenticate_header_equality_with_different_parameters_order() {
        header_equality(
            r#"WWW-Authenticate: Digest realm="atlanta.com", nextnonce="47364c23432d2e131a5fb210812c""#,
            r#"WWW-Authenticate: Digest nextnonce="47364c23432d2e131a5fb210812c", realm="atlanta.com""#,
        );
    }

    #[test]
    fn test_www_authenticate_header_equality_with_different_qop_options_order() {
        header_equality(
            r#"WWW-Authenticate: Digest qop="auth,auth-int""#,
            r#"WWW-Authenticate: Digest qop="auth-int,auth""#,
        );
    }

    #[test]
    fn test_www_authenticate_header_equality_with_different_cases_1() {
        header_equality(
            "WWW-Authenticate: Digest stale=true",
            "WWW-authenticate: digest  STALE=True",
        );
    }

    #[test]
    fn test_www_authenticate_header_equality_with_different_cases_2() {
        header_equality(
            "WWW-Authenticate: CustomScheme algorithm=MD5-Sess",
            "WWW-Authenticate: customscheme  Algorithm=Md5-sess",
        );
    }

    #[test]
    fn test_www_authenticate_header_inequality_with_different_parameter_values() {
        header_inequality(
            r#"WWW-Authenticate: Digest qop="auth""#,
            r#"WWW-Authenticate: Digest qop="auth-int""#,
        );
    }

    #[test]
    fn test_www_authenticate_header_inequality_with_different_parameters() {
        header_inequality(
            r#"WWW-Authenticate: Digest qop="auth""#,
            r#"WWW-Authenticate: Digest nextnonce="47364c23432d2e131a5fb210812c""#,
        );
    }

    #[test]
    fn test_www_authenticate_header_inequality_with_different_schemes() {
        header_inequality(
            "WWW-Authenticate: Digest algorithm=MD5",
            "WWW-Authenticate: CustomScheme algorithm=MD5",
        );
    }

    #[test]
    fn test_www_authenticate_header_to_string() {
        let header = Header::try_from(
            r#"WwW-AuthenticatE  :    Digest   realm="atlanta.com", domain = "sip:ss1.carrier.com", qop="auth", nonce=  "f84f1cec41e6cbe5aea9c8e88d359"  , opaque="", stale  =FALSE, algorithm=MD5"#,
        );
        if let Header::WWWAuthenticate(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"WwW-AuthenticatE  :    Digest   realm="atlanta.com", domain = "sip:ss1.carrier.com", qop="auth", nonce=  "f84f1cec41e6cbe5aea9c8e88d359"  , opaque="", stale  =FALSE, algorithm=MD5"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"WWW-Authenticate: Digest realm="atlanta.com", domain="sip:ss1.carrier.com", qop="auth", nonce="f84f1cec41e6cbe5aea9c8e88d359", opaque="", stale=FALSE, algorithm=MD5"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"WWW-Authenticate: Digest realm="atlanta.com", domain="sip:ss1.carrier.com", qop="auth", nonce="f84f1cec41e6cbe5aea9c8e88d359", opaque="", stale=FALSE, algorithm=MD5"#
            );
        }
    }
}
