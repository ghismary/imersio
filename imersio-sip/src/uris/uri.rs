//! Parsing and generation of a URI and its parts, either a SIP URI or an absolute URI.

use crate::{AbsoluteUri, Host, SipError, SipUri, UriHeaders, UriParameters, UriScheme};
use nom::error::convert_error;
use std::convert::TryFrom;

/// Representation of a URI, whether a SIP URI or an absolute URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Uri {
    /// A sip: or sips: URI
    Sip(SipUri),
    /// Any other URI
    Absolute(AbsoluteUri),
}

impl Uri {
    /// Get the `Uri` as an `AbsoluteUri`.
    ///
    /// It returns None if the uri is not an `AbsoluteUri`.
    pub fn as_absolute_uri(&self) -> Option<&AbsoluteUri> {
        match self {
            Uri::Absolute(uri) => Some(uri),
            _ => None,
        }
    }

    /// Get the `Uri` as a `SipUri`.
    ///
    /// It returns None if the uri is not a `SipUri`.
    pub fn as_sip_uri(&self) -> Option<&SipUri> {
        match self {
            Uri::Sip(uri) => Some(uri),
            _ => None,
        }
    }

    /// Tell whether this `Uri` is a SIP URI.
    pub fn is_sip(&self) -> bool {
        matches!(self, Uri::Sip(_))
    }

    /// Telss whether this `Uri` is secure or not.
    pub fn is_secure(&self) -> bool {
        match self {
            Uri::Sip(uri) => uri.scheme() == &UriScheme::SIPS,
            _ => false,
        }
    }

    /// Get the `Scheme` of the URI.
    pub fn scheme(&self) -> &UriScheme {
        match self {
            Uri::Sip(uri) => uri.scheme(),
            Uri::Absolute(uri) => uri.scheme(),
        }
    }

    /// Get the user from the URI.
    pub fn user(&self) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.userinfo().map(|ui| ui.get_user()),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the password from the URI.
    pub fn password(&self) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.userinfo().and_then(|ui| ui.get_password()),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the host from the URI.
    pub fn host(&self) -> Option<&Host> {
        match self {
            Uri::Sip(uri) => Some(uri.host()),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the port from the URI.
    pub fn port(&self) -> Option<u16> {
        match self {
            Uri::Sip(uri) => uri.port(),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the `Parameters` of the URI.
    pub fn parameters(&self) -> &UriParameters {
        match self {
            Uri::Sip(uri) => uri.parameters(),
            Uri::Absolute(uri) => uri.parameters(),
        }
    }

    /// Get a parameter value of the URI given its name.
    pub fn parameter(&self, name: &str) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.parameters().get(name).and_then(|p| p.value()),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the `Headers` of the URI.
    pub fn headers(&self) -> &UriHeaders {
        match self {
            Uri::Sip(uri) => uri.headers(),
            Uri::Absolute(uri) => uri.headers(),
        }
    }

    /// Get a header value of the URI given its name.
    pub fn header(&self, name: &str) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.headers().get(name),
            Uri::Absolute(_) => None,
        }
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Uri::Sip(uri) => uri.to_string(),
                Uri::Absolute(uri) => uri.to_string(),
            }
        )
    }
}

impl TryFrom<&str> for Uri {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::request_uri(value) {
            Ok((rest, uri)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(uri)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidUri(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => {
                Err(SipError::InvalidUri(format!("Incomplete uri `{}`", value)))
            }
        }
    }
}

impl Default for Uri {
    fn default() -> Self {
        Uri::Sip(SipUri::default())
    }
}

impl PartialEq<&Uri> for Uri {
    fn eq(&self, other: &&Uri) -> bool {
        self == *other
    }
}

impl PartialEq<Uri> for &Uri {
    fn eq(&self, other: &Uri) -> bool {
        *self == other
    }
}

pub(crate) mod parser {
    use crate::parser::ParserResult;
    use crate::uris::absolute_uri::parser::absolute_uri;
    use crate::uris::sip_uri::parser::sip_uri;
    use crate::Uri;
    use nom::{branch::alt, combinator::map, error::context};

    pub(crate) fn request_uri(input: &str) -> ParserResult<&str, Uri> {
        context("uri", alt((sip_uri, map(absolute_uri, Uri::Absolute))))(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_valid_sip_uri_without_parameters_and_without_headers() {
        let uri = Uri::try_from("sip:alice@atlanta.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIP);
        assert_eq!(uri.user(), Some("alice"));
        assert!(uri.password().is_none());
        assert_eq!(uri.host(), Some(&Host::Name("atlanta.com".to_string())));
        assert!(uri.port().is_none());
        assert!(uri.parameters().is_empty());
        assert!(uri.headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice@atlanta.com");
    }

    #[test]
    fn test_valid_sip_uri_with_parameter() {
        let uri = Uri::try_from("sip:alice:secretword@atlanta.com;transport=tcp");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIP);
        assert_eq!(uri.user(), Some("alice"));
        assert_eq!(uri.password(), Some("secretword"));
        assert_eq!(uri.host(), Some(&Host::Name("atlanta.com".to_string())));
        assert!(uri.port().is_none());
        assert_eq!(uri.parameters().len(), 1);
        assert_eq!(uri.parameter("transport"), Some("tcp"));
        assert!(uri.headers().is_empty());
        assert_eq!(
            uri.to_string(),
            "sip:alice:secretword@atlanta.com;transport=tcp"
        );
    }

    #[test]
    fn test_valid_sip_uri_with_headers() {
        let uri = Uri::try_from("sips:alice@atlanta.com?subject=project%20x&priority=urgent");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIPS);
        assert_eq!(uri.user(), Some("alice"));
        assert!(uri.password().is_none());
        assert_eq!(uri.host(), Some(&Host::Name("atlanta.com".to_string())));
        assert!(uri.port().is_none());
        assert!(uri.parameters().is_empty());
        assert_eq!(uri.headers().len(), 2);
        assert_eq!(uri.header("subject"), Some("project x"));
        assert_eq!(uri.header("priority"), Some("urgent"));
        assert_eq!(
            uri.to_string(),
            "sips:alice@atlanta.com?subject=project%20x&priority=urgent"
        );
    }

    #[test]
    fn test_valid_sip_uri_with_password_and_parameter() {
        let uri = Uri::try_from("sip:+1-212-555-1212:1234@gateway.com;user=phone");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIP);
        assert_eq!(uri.user(), Some("+1-212-555-1212"));
        assert_eq!(uri.password(), Some("1234"));
        assert_eq!(uri.host(), Some(&Host::Name("gateway.com".to_string())));
        assert!(uri.port().is_none());
        assert_eq!(uri.parameters().len(), 1);
        assert_eq!(uri.parameter("user"), Some("phone"));
        assert!(uri.headers().is_empty());
        assert_eq!(
            uri.to_string(),
            "sip:+1-212-555-1212:1234@gateway.com;user=phone"
        );
    }

    #[test]
    fn test_valid_sips_uri_without_parameters_and_without_headers() {
        let uri = Uri::try_from("sips:1212@gateway.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIPS);
        assert_eq!(uri.user(), Some("1212"));
        assert!(uri.password().is_none());
        assert_eq!(uri.host(), Some(&Host::Name("gateway.com".to_string())));
        assert!(uri.port().is_none());
        assert!(uri.parameters().is_empty());
        assert!(uri.headers().is_empty());
        assert_eq!(uri.to_string(), "sips:1212@gateway.com");
    }

    #[test]
    fn test_valid_sip_uri_with_ipv4_address() {
        let uri = Uri::try_from("sip:alice@192.0.2.4");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIP);
        assert_eq!(uri.user(), Some("alice"));
        assert!(uri.password().is_none());
        assert_eq!(
            uri.host(),
            Some(&Host::Ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 4))))
        );
        assert!(uri.port().is_none());
        assert!(uri.parameters().is_empty());
        assert!(uri.headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice@192.0.2.4");
    }

    #[test]
    fn test_valid_sip_uri_with_parameter_and_header() {
        let uri = Uri::try_from("sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIP);
        assert!(uri.user().is_none());
        assert!(uri.password().is_none());
        assert_eq!(uri.host(), Some(&Host::Name("atlanta.com".to_string())));
        assert!(uri.port().is_none());
        assert_eq!(uri.parameters().len(), 1);
        assert_eq!(uri.parameter("method"), Some("REGISTER"));
        assert_eq!(uri.headers().len(), 1);
        assert_eq!(uri.header("to"), Some("alice@atlanta.com"));
        assert_eq!(
            uri.to_string(),
            "sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com"
        );
    }

    #[test]
    fn test_valid_sip_uri_without_username_and_with_parameter() {
        let uri = Uri::try_from("sip:alice;day=tuesday@atlanta.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIP);
        assert_eq!(uri.user(), Some("alice;day=tuesday"));
        assert!(uri.password().is_none());
        assert_eq!(uri.host(), Some(&Host::Name("atlanta.com".to_string())));
        assert!(uri.port().is_none());
        assert!(uri.parameters().is_empty());
        assert!(uri.headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice;day=tuesday@atlanta.com");
    }

    #[test]
    fn test_valid_sip_uri_with_escaped_character_in_password() {
        let uri = Uri::try_from("sip:alice:secret%77ord@atlanta.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIP);
        assert_eq!(uri.user(), Some("alice"));
        assert_eq!(uri.password(), Some("secretword"));
        assert_eq!(uri.host(), Some(&Host::Name("atlanta.com".to_string())));
        assert!(uri.port().is_none());
        assert!(uri.parameters().is_empty());
        assert!(uri.headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice:secretword@atlanta.com");
    }

    #[test]
    fn test_valid_sip_uri_with_escaped_characters_in_parameter() {
        // Check escaped chars in parameters.
        let uri = Uri::try_from("sip:alice:secretword@atlanta.com;%74ransport=t%63p");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIP);
        assert_eq!(uri.user(), Some("alice"));
        assert_eq!(uri.password(), Some("secretword"));
        assert_eq!(uri.host(), Some(&Host::Name("atlanta.com".to_string())));
        assert!(uri.port().is_none());
        assert_eq!(uri.parameters().len(), 1);
        assert_eq!(uri.parameter("transport"), Some("tcp"));
        assert!(uri.headers().is_empty());
        assert_eq!(
            uri.to_string(),
            "sip:alice:secretword@atlanta.com;transport=tcp"
        );
    }

    #[test]
    fn test_valid_sip_uri_with_escaped_characters_in_header() {
        let uri = Uri::try_from("sip:atlanta.com;method=REGISTER?t%6f=al%69ce%40atlant%61.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.scheme(), &UriScheme::SIP);
        assert!(uri.user().is_none());
        assert!(uri.password().is_none());
        assert_eq!(uri.host(), Some(&Host::Name("atlanta.com".to_string())));
        assert!(uri.port().is_none());
        assert_eq!(uri.parameters().len(), 1);
        assert_eq!(uri.parameter("method"), Some("REGISTER"));
        assert_eq!(uri.headers().len(), 1);
        assert_eq!(uri.header("to"), Some("alice@atlanta.com"));
        assert_eq!(
            uri.to_string(),
            "sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com"
        );
    }

    #[test]
    fn test_invalid_sip_uri_multiple_parameters_with_the_same_name() {
        assert_err!(Uri::try_from(
            "sip:alice@atlanta.com;transport=tcp;transport=udp"
        ));
    }

    #[test]
    fn test_invalid_sip_uri_invalid_ipv4_address() {
        assert_err!(Uri::try_from("sip:alice@1923.0.2.4"));
    }

    #[test]
    fn test_invalid_sip_uri_invalid_ipv4_address_2() {
        assert_err!(Uri::try_from("sip:alice@192.0.329.18"));
    }

    #[test]
    fn test_invalid_sip_uri_multiple_at_characters() {
        assert_err!(Uri::try_from("sip:alice@atlanta.com@gateway.com"));
    }

    #[test]
    fn test_sip_uri_equality_case_differences() {
        assert_eq!(
            Uri::try_from("sip:%61lice@atlanta.com;transport=TCP").unwrap(),
            Uri::try_from("sip:alice@AtLanTa.CoM;Transport=tcp").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_equality_one_with_a_parameter_the_other_without() {
        assert_eq!(
            Uri::try_from("sip:carol@chicago.com").unwrap(),
            Uri::try_from("sip:carol@chicago.com;newparam=5").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_equality_one_with_a_parameter_the_other_without_2() {
        assert_eq!(
            Uri::try_from("sip:carol@chicago.com").unwrap(),
            Uri::try_from("sip:carol@chicago.com;security=on").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_equality_different_parameters() {
        assert_eq!(
            Uri::try_from("sip:carol@chicago.com;newparam=5").unwrap(),
            Uri::try_from("sip:carol@chicago.com;security=on").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_equality_same_parameters_in_different_order() {
        assert_eq!(
            Uri::try_from("sip:biloxi.com;transport=tcp;method=REGISTER?to=sip:bob%40biloxi.com")
                .unwrap(),
            Uri::try_from("sip:biloxi.com;method=REGISTER;transport=tcp?to=sip:bob%40biloxi.com")
                .unwrap()
        );
    }

    #[test]
    fn test_sip_uri_equality_same_headers_in_different_order() {
        assert_eq!(
            Uri::try_from("sip:alice@atlanta.com?subject=project%20x&priority=urgent").unwrap(),
            Uri::try_from("sip:alice@atlanta.com?priority=urgent&subject=project%20x").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_inequality_different_cases() {
        assert_ne!(
            Uri::try_from("SIP:ALICE@AtLanTa.CoM;Transport=udp").unwrap(),
            Uri::try_from("sip:alice@AtLanTa.CoM;Transport=UDP").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_inequality_one_with_a_port_the_other_without() {
        assert_ne!(
            Uri::try_from("sip:bob@biloxi.com").unwrap(),
            Uri::try_from("sip:bob@biloxi.com:5060").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_inequality_one_with_transport_parameter_the_other_without() {
        assert_ne!(
            Uri::try_from("sip:bob@biloxi.com").unwrap(),
            Uri::try_from("sip:bob@biloxi.com;transport=udp").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_inequality_one_with_transport_parameter_and_port_the_other_without() {
        assert_ne!(
            Uri::try_from("sip:bob@biloxi.com").unwrap(),
            Uri::try_from("sip:bob@biloxi.com:6000;transport=tcp").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_inequality_one_with_subject_header_the_other_without() {
        assert_ne!(
            Uri::try_from("sip:carol@chicago.com").unwrap(),
            Uri::try_from("sip:carol@chicago.com?Subject=next%20meeting").unwrap()
        );
    }

    #[test]
    fn test_sip_uri_inequality_one_with_hostname_the_other_with_ipv4_address() {
        assert_ne!(
            Uri::try_from("sip:bob@phone21.boxesbybob.com").unwrap(),
            Uri::try_from("sip:bob@192.0.2.4").unwrap()
        );
    }

    #[test]
    fn test_uris_intransitivity() {
        assert_eq!(
            Uri::try_from("sip:carol@chicago.com").unwrap(),
            Uri::try_from("sip:carol@chicago.com;security=on").unwrap()
        );
        assert_eq!(
            Uri::try_from("sip:carol@chicago.com").unwrap(),
            Uri::try_from("sip:carol@chicago.com;security=off").unwrap()
        );
        assert_ne!(
            Uri::try_from("sip:carol@chicago.com;security=on").unwrap(),
            Uri::try_from("sip:carol@chicago.com;security=off").unwrap()
        );
    }
}
