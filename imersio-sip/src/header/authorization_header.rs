use std::{collections::HashSet, hash::Hash};

use crate::{
    common::{Algorithm, MessageQop},
    Error, Uri,
};

use super::authentication_info_header::AInfo;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationHeader(Credentials);

impl AuthorizationHeader {
    pub(crate) fn new(credentials: Credentials) -> Self {
        AuthorizationHeader(credentials)
    }

    /// Get a reference to the `Credentials` of the Authorization header.
    pub fn credentials(&self) -> &Credentials {
        &self.0
    }
}

impl std::fmt::Display for AuthorizationHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Authorization: {}", self.0)
    }
}

impl PartialEq<&AuthorizationHeader> for AuthorizationHeader {
    fn eq(&self, other: &&AuthorizationHeader) -> bool {
        self == *other
    }
}

impl PartialEq<AuthorizationHeader> for &AuthorizationHeader {
    fn eq(&self, other: &AuthorizationHeader) -> bool {
        *self == other
    }
}

#[derive(Clone, Debug)]
pub enum Credentials {
    Digest(Vec<AuthParameter>),
    Other(String, Vec<AuthParameter>),
}

impl Credentials {
    /// Get the number of `AuthParam` in the Credentials.
    pub fn count(&self) -> usize {
        match self {
            Self::Digest(params) => params.len(),
            Self::Other(_, params) => params.len(),
        }
    }

    /// Tells whether Authorization header contains the given authorization
    /// parameter key.
    pub fn contains(&self, key: &str) -> bool {
        self.auth_params().iter().any(|p| p.key() == key)
    }

    /// Gets the `AuthParam` corresponding to the given authorization
    /// parameter key.
    pub fn get(&self, key: &str) -> Option<&AuthParameter> {
        self.auth_params().iter().find(|p| p.key() == key)
    }

    /// Tells whether the `Credentials` is a Digest.
    pub fn is_digest(&self) -> bool {
        matches!(self, Self::Digest(_))
    }

    /// Get the scheme of the Credentials.
    pub fn scheme(&self) -> &str {
        match self {
            Self::Digest(_) => "Digest",
            Self::Other(scheme, _) => scheme,
        }
    }

    /// Get a reference to the `AuthParam`s in the Credentials.
    pub fn auth_params(&self) -> &Vec<AuthParameter> {
        match self {
            Self::Digest(params) => params,
            Self::Other(_, params) => params,
        }
    }

    /// Tells whether the Authorization header contains a `algorithm` value.
    pub fn has_algorithm(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::Algorithm(_))),
            _ => false,
        }
    }

    /// Get the `algorithm` value from the Authorization header.
    pub fn algorithm(&self) -> Option<&Algorithm> {
        match self {
            Self::Digest(params) => params
                .iter()
                .find(|param| matches!(param, AuthParameter::Algorithm(_)))
                .and_then(|param| {
                    if let AuthParameter::Algorithm(value) = param {
                        Some(value)
                    } else {
                        None
                    }
                }),
            _ => None,
        }
    }

    /// Tells whether the Authorization header contains a `uri` value.
    pub fn has_digest_uri(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::DigestUri(_))),
            _ => false,
        }
    }

    /// Get the `uri` value from the Authorization header.
    pub fn digest_uri(&self) -> Option<&Uri> {
        match self {
            Self::Digest(params) => params
                .iter()
                .find(|param| matches!(param, AuthParameter::DigestUri(_)))
                .and_then(|param| {
                    if let AuthParameter::DigestUri(value) = param {
                        Some(value)
                    } else {
                        None
                    }
                }),
            _ => None,
        }
    }

    /// Tells whether the Authorization header contains a `qop` value.
    pub fn has_message_qop(&self) -> bool {
        match self {
            Self::Digest(params) => params
                .iter()
                .any(|param| matches!(param, AuthParameter::MessageQop(_))),
            _ => false,
        }
    }

    /// Get the `qop` value from the Authorization header.
    pub fn message_qop(&self) -> Option<&MessageQop> {
        match self {
            Self::Digest(params) => params
                .iter()
                .find(|param| matches!(param, AuthParameter::MessageQop(_)))
                .and_then(|param| {
                    if let AuthParameter::MessageQop(value) = param {
                        Some(value)
                    } else {
                        None
                    }
                }),
            _ => None,
        }
    }
}

impl std::fmt::Display for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (scheme, params) = match self {
            Self::Digest(params) => ("Digest".to_string(), params),
            Self::Other(scheme, params) => (scheme.clone(), params),
        };

        write!(
            f,
            "{} {}",
            scheme,
            params
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl PartialEq for Credentials {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Digest(self_params), Self::Digest(other_params)) => {
                let self_params: HashSet<_> = self_params.iter().collect();
                let other_params: HashSet<_> = other_params.iter().collect();
                self_params == other_params
            }
            (Self::Other(self_scheme, self_params), Self::Other(other_scheme, other_params)) => {
                if !self_scheme.eq_ignore_ascii_case(other_scheme) {
                    false
                } else {
                    let self_params: HashSet<_> = self_params.iter().collect();
                    let other_params: HashSet<_> = other_params.iter().collect();
                    self_params == other_params
                }
            }
            _ => false,
        }
    }
}

impl PartialEq<&Credentials> for Credentials {
    fn eq(&self, other: &&Credentials) -> bool {
        self == *other
    }
}

impl PartialEq<Credentials> for &Credentials {
    fn eq(&self, other: &Credentials) -> bool {
        *self == other
    }
}

impl Eq for Credentials {}

macro_rules! credentials {
    (
        $(
            ($token:ident, $has_token:ident, $enum_name:ident);
        )+
    ) => {
        impl Credentials {
            $(
                /// Tells whether the Authorization header contains a `$token` value.
                pub fn $has_token(&self) -> bool {
                    match self {
                        Self::Digest(params) => params.iter().any(|param| matches!(param, AuthParameter::$enum_name(_))),
                        _ => false
                    }
                }

                /// Get the `$token` value from the Authorization header.
                pub fn $token(&self) -> Option<&str> {
                    match self {
                        Self::Digest(params) => params
                        .iter()
                        .find(|param| matches!(param, AuthParameter::$enum_name(_)))
                        .map(|param| {
                            if let AuthParameter::$enum_name(value) = param {
                                value
                            } else {
                                ""
                            }
                        }),
                        _ => None
                    }
                }
            )+
        }
    }
}

credentials! {
    (username, has_username, Username);
    (realm, has_realm, Realm);
    (nonce, has_nonce, Nonce);
    (dresponse, has_dresponse, DResponse);
    (cnonce, has_cnonce, CNonce);
    (opaque, has_opaque, Opaque);
    (nonce_count, has_nonce_count, NonceCount);
}

#[derive(Clone, Debug)]
pub enum AuthParameter {
    Username(String),
    Realm(String),
    Nonce(String),
    DigestUri(Uri),
    DResponse(String),
    Algorithm(Algorithm),
    CNonce(String),
    Opaque(String),
    MessageQop(MessageQop),
    NonceCount(String),
    Other(String, String),
}

impl AuthParameter {
    pub fn key(&self) -> &str {
        match self {
            Self::Username(_) => "username",
            Self::Realm(_) => "realm",
            Self::Nonce(_) => "nonce",
            Self::DigestUri(_) => "uri",
            Self::DResponse(_) => "response",
            Self::Algorithm(_) => "algorithm",
            Self::CNonce(_) => "cnonce",
            Self::Opaque(_) => "opaque",
            Self::MessageQop(_) => "qop",
            Self::NonceCount(_) => "nc",
            Self::Other(key, _) => key,
        }
    }

    pub fn value(&self) -> String {
        match self {
            Self::Username(value) => value.clone(),
            Self::Realm(value) => value.clone(),
            Self::Nonce(value) => value.clone(),
            Self::DigestUri(value) => value.to_string(),
            Self::DResponse(value) => value.clone(),
            Self::Algorithm(value) => value.value().to_string(),
            Self::CNonce(value) => value.clone(),
            Self::Opaque(value) => value.clone(),
            Self::MessageQop(value) => value.value().to_string(),
            Self::NonceCount(value) => value.clone(),
            Self::Other(_, value) => value.clone(),
        }
    }
}

impl std::fmt::Display for AuthParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (key, value) = match self {
            Self::Username(value) => ("username".to_string(), format!("\"{value}\"")),
            Self::Realm(value) => ("realm".to_string(), format!("\"{value}\"")),
            Self::Nonce(value) => ("nonce".to_string(), format!("\"{value}\"")),
            Self::DigestUri(value) => ("uri".to_string(), format!("\"{value}\"")),
            Self::DResponse(value) => ("response".to_string(), format!("\"{value}\"")),
            Self::Algorithm(value) => ("algorithm".to_string(), value.to_string()),
            Self::CNonce(value) => ("cnonce".to_string(), format!("\"{value}\"")),
            Self::Opaque(value) => ("opaque".to_string(), format!("\"{value}\"")),
            Self::MessageQop(value) => ("qop".to_string(), value.value().to_string()),
            Self::NonceCount(value) => ("nc".to_string(), value.clone()),
            Self::Other(key, value) => (key.clone(), value.clone()),
        };
        write!(f, "{}={}", key, value)
    }
}

impl PartialEq<AuthParameter> for AuthParameter {
    fn eq(&self, other: &AuthParameter) -> bool {
        match (self, other) {
            (Self::Username(a), Self::Username(b))
            | (Self::Realm(a), Self::Realm(b))
            | (Self::Nonce(a), Self::Nonce(b))
            | (Self::DResponse(a), Self::DResponse(b))
            | (Self::CNonce(a), Self::CNonce(b))
            | (Self::Opaque(a), Self::Opaque(b)) => a == b,
            (Self::DigestUri(a), Self::DigestUri(b)) => a == b,
            (Self::Algorithm(a), Self::Algorithm(b)) => a == b,
            (Self::MessageQop(a), Self::MessageQop(b)) => a == b,
            (Self::NonceCount(a), Self::NonceCount(b)) => a.eq_ignore_ascii_case(b),
            (Self::Other(akey, avalue), Self::Other(bkey, bvalue)) => {
                akey.eq_ignore_ascii_case(bkey) && avalue.eq_ignore_ascii_case(bvalue)
            }
            _ => false,
        }
    }
}

impl PartialEq<&AuthParameter> for AuthParameter {
    fn eq(&self, other: &&AuthParameter) -> bool {
        self == *other
    }
}

impl PartialEq<AuthParameter> for &AuthParameter {
    fn eq(&self, other: &AuthParameter) -> bool {
        *self == other
    }
}

impl Eq for AuthParameter {}

impl Hash for AuthParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        match self {
            Self::Username(value)
            | Self::Realm(value)
            | Self::Nonce(value)
            | Self::DResponse(value)
            | Self::CNonce(value)
            | Self::Opaque(value) => value.hash(state),
            Self::DigestUri(value) => value.to_string().hash(state),
            Self::Algorithm(value) => value.value().to_ascii_lowercase().hash(state),
            Self::MessageQop(value) => value.value().to_ascii_lowercase().hash(state),
            Self::NonceCount(value) => value.to_ascii_lowercase().hash(state),
            Self::Other(_, value) => value.to_ascii_lowercase().hash(state),
        }
    }
}

impl TryFrom<AInfo> for AuthParameter {
    type Error = Error;

    fn try_from(value: AInfo) -> Result<Self, Self::Error> {
        match value {
            AInfo::CNonce(value) => Ok(AuthParameter::CNonce(value)),
            AInfo::MessageQop(value) => Ok(AuthParameter::MessageQop(value)),
            AInfo::NonceCount(value) => Ok(AuthParameter::NonceCount(value)),
            _ => Err(Error::FailedConvertingAInfoToAuthParam),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthorizationHeader;
    use crate::{
        common::{Algorithm, MessageQop},
        header::authorization_header::AuthParameter,
        Header, Uri,
    };
    use std::str::FromStr;

    fn valid_header<F: FnOnce(AuthorizationHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert!(header.is_ok());
        if let Header::Authorization(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not an Authorization header");
        }
    }

    #[test]
    fn test_valid_authorization_header_1() {
        valid_header(
            r#"Authorization: Digest username="Alice", realm="atlanta.com", nonce="84a4cc6f3082121f32b42a2187831a9e", response="7587245234b3434cc3412213e5f113a5""#,
            |header| {
                let credentials = header.credentials();
                assert_eq!(credentials.scheme(), "Digest");
                assert_eq!(credentials.count(), 4);
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
                assert_eq!(credentials.count(), 9);
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
                    Uri::from_str("sip:bob@biloxi.com").unwrap()
                );
                assert!(credentials.has_message_qop());
                assert_eq!(credentials.message_qop(), Some(&MessageQop::Auth));
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
                credentials.auth_params().first().unwrap(),
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

    fn invalid_header(header: &str) {
        assert!(Header::from_str(header).is_err());
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

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::Authorization(first_header), Header::Authorization(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Authorization header");
        }
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

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::Authorization(first_header), Header::Authorization(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Authorization header");
        }
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
}
