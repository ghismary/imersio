//! TODO

mod absolute_uri;
mod host_port;
pub(crate) mod parser;
mod sip_uri;
mod uri_headers;
mod uri_parameters;
mod uri_scheme;
mod user_info;

pub use absolute_uri::AbsoluteUri;
pub use host_port::HostPort;
pub use sip_uri::SipUri;
pub use uri_headers::UriHeaders;
pub use uri_parameters::UriParameters;
pub use uri_scheme::UriScheme;
pub use user_info::UserInfo;

use std::{hash::Hash, num::NonZeroU16, str::FromStr};

use crate::Error;

/// Representation of an URI, whether a SIP URI or an absolute URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Uri {
    /// A sip: or sips: URI
    Sip(SipUri),
    /// Any other URI
    Absolute(AbsoluteUri),
}

impl Uri {
    /// Try to create a `Uri` from a slice of bytes.
    #[inline]
    pub fn from_bytes(input: &[u8]) -> Result<Uri, Error> {
        parse_uri(input)
    }

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
            Uri::Sip(uri) => uri.scheme() == UriScheme::SIPS,
            _ => false,
        }
    }

    /// Get the `Scheme` of the URI.
    pub fn get_scheme(&self) -> &UriScheme {
        match self {
            Uri::Sip(uri) => uri.scheme(),
            Uri::Absolute(uri) => uri.scheme(),
        }
    }

    /// Get the user from the URI.
    pub fn get_user(&self) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.userinfo().map(|ui| ui.get_user()),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the password from the URI.
    pub fn get_password(&self) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.userinfo().and_then(|ui| ui.get_password()),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the host from the URI.
    pub fn get_host(&self) -> &str {
        match self {
            Uri::Sip(uri) => uri.hostport().get_host(),
            Uri::Absolute(uri) => uri.opaque_part(),
        }
    }

    /// Get the port from the URI.
    pub fn get_port(&self) -> Option<NonZeroU16> {
        match self {
            Uri::Sip(uri) => uri.hostport().get_port(),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the `Parameters` of the URI.
    pub fn get_parameters(&self) -> &UriParameters {
        match self {
            Uri::Sip(uri) => uri.parameters(),
            Uri::Absolute(uri) => uri.parameters(),
        }
    }

    /// Get a parameter value of the URI given its name.
    pub fn get_parameter(&self, name: &str) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.parameters().get(name),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the `Headers` of the URI.
    pub fn get_headers(&self) -> &UriHeaders {
        match self {
            Uri::Sip(uri) => uri.headers(),
            Uri::Absolute(uri) => uri.headers(),
        }
    }

    /// Get a header value of the URI given its name.
    pub fn get_header(&self, name: &str) -> Option<&str> {
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

impl FromStr for Uri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uri::from_bytes(s.as_bytes())
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

fn parse_uri(input: &[u8]) -> Result<Uri, Error> {
    match parser::request_uri(input) {
        Ok((rest, uri)) => {
            if !rest.is_empty() {
                Err(Error::RemainingUnparsedData)
            } else {
                Ok(uri)
            }
        }
        Err(e) => Err(Error::InvalidUri(e.to_string())),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_valid_uri_parsing() {
        let uri = Uri::from_str("sip:alice@atlanta.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice@atlanta.com");

        let uri = Uri::from_str("sip:alice:secretword@atlanta.com;transport=tcp");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert_eq!(uri.get_password(), Some("secretword"));
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("transport"), Some("tcp"));
        assert!(uri.get_headers().is_empty());
        assert_eq!(
            uri.to_string(),
            "sip:alice:secretword@atlanta.com;transport=tcp"
        );

        let uri = Uri::from_str("sips:alice@atlanta.com?subject=project%20x&priority=urgent");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIPS);
        assert_eq!(uri.get_user(), Some("alice"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert_eq!(uri.get_headers().len(), 2);
        assert_eq!(uri.get_header("subject"), Some("project x"));
        assert_eq!(uri.get_header("priority"), Some("urgent"));
        assert_eq!(
            uri.to_string(),
            "sips:alice@atlanta.com?subject=project%20x&priority=urgent"
        );

        let uri = Uri::from_str("sip:+1-212-555-1212:1234@gateway.com;user=phone");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIP);
        assert_eq!(uri.get_user(), Some("+1-212-555-1212"));
        assert_eq!(uri.get_password(), Some("1234"));
        assert_eq!(uri.get_host(), "gateway.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("user"), Some("phone"));
        assert!(uri.get_headers().is_empty());
        assert_eq!(
            uri.to_string(),
            "sip:+1-212-555-1212:1234@gateway.com;user=phone"
        );

        let uri = Uri::from_str("sips:1212@gateway.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIPS);
        assert_eq!(uri.get_user(), Some("1212"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "gateway.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sips:1212@gateway.com");

        let uri = Uri::from_str("sip:alice@192.0.2.4");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "192.0.2.4");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice@192.0.2.4");

        let uri = Uri::from_str("sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIP);
        assert!(uri.get_user().is_none());
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("method"), Some("REGISTER"));
        assert_eq!(uri.get_headers().len(), 1);
        assert_eq!(uri.get_header("to"), Some("alice@atlanta.com"));
        assert_eq!(
            uri.to_string(),
            "sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com"
        );

        let uri = Uri::from_str("sip:alice;day=tuesday@atlanta.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIP);
        assert_eq!(uri.get_user(), Some("alice;day=tuesday"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice;day=tuesday@atlanta.com");

        // Check escaped char in password.
        let uri = Uri::from_str("sip:alice:secret%77ord@atlanta.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert_eq!(uri.get_password(), Some("secretword"));
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice:secretword@atlanta.com");

        // Check escaped chars in parameters.
        let uri = Uri::from_str("sip:alice:secretword@atlanta.com;%74ransport=t%63p");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert_eq!(uri.get_password(), Some("secretword"));
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("transport"), Some("tcp"));
        assert!(uri.get_headers().is_empty());
        assert_eq!(
            uri.to_string(),
            "sip:alice:secretword@atlanta.com;transport=tcp"
        );

        // Check escaped chars in headers.
        let uri = Uri::from_str("sip:atlanta.com;method=REGISTER?t%6f=al%69ce%40atlant%61.com");
        assert_ok!(&uri);
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), UriScheme::SIP);
        assert!(uri.get_user().is_none());
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("method"), Some("REGISTER"));
        assert_eq!(uri.get_headers().len(), 1);
        assert_eq!(uri.get_header("to"), Some("alice@atlanta.com"));
        assert_eq!(
            uri.to_string(),
            "sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com"
        );
    }

    #[test]
    fn test_invalid_uri_parsing() {
        // Multiple parameters with the same name are not allowed.
        let uri = Uri::from_str("sip:alice@atlanta.com;transport=tcp;transport=udp");
        assert_err!(uri);

        // Invalid IPv4 address.
        let uri = Uri::from_str("sip:alice@1923.0.2.4");
        assert_err!(uri);
        let uri = Uri::from_str("sip:alice@192.0.329.18");
        assert_err!(uri);

        // Invalid multiple `@` characters.
        let uri = Uri::from_str("sip:alice@atlanta.com@gateway.com");
        assert_err!(uri);
    }

    #[test]
    fn test_uris_equal() {
        assert_eq!(
            Uri::from_str("sip:%61lice@atlanta.com;transport=TCP").unwrap(),
            Uri::from_str("sip:alice@AtLanTa.CoM;Transport=tcp").unwrap()
        );

        assert_eq!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com;newparam=5").unwrap()
        );
        assert_eq!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=on").unwrap()
        );
        assert_eq!(
            Uri::from_str("sip:carol@chicago.com;newparam=5").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=on").unwrap()
        );

        assert_eq!(
            Uri::from_str("sip:biloxi.com;transport=tcp;method=REGISTER?to=sip:bob%40biloxi.com")
                .unwrap(),
            Uri::from_str("sip:biloxi.com;method=REGISTER;transport=tcp?to=sip:bob%40biloxi.com")
                .unwrap()
        );

        assert_eq!(
            Uri::from_str("sip:alice@atlanta.com?subject=project%20x&priority=urgent").unwrap(),
            Uri::from_str("sip:alice@atlanta.com?priority=urgent&subject=project%20x").unwrap()
        );
    }

    #[test]
    fn test_uris_not_equal() {
        assert_ne!(
            Uri::from_str("SIP:ALICE@AtLanTa.CoM;Transport=udp").unwrap(),
            Uri::from_str("sip:alice@AtLanTa.CoM;Transport=UDP").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:bob@biloxi.com").unwrap(),
            Uri::from_str("sip:bob@biloxi.com:5060").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:bob@biloxi.com").unwrap(),
            Uri::from_str("sip:bob@biloxi.com;transport=udp").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:bob@biloxi.com").unwrap(),
            Uri::from_str("sip:bob@biloxi.com:6000;transport=tcp").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com?Subject=next%20meeting").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:bob@phone21.boxesbybob.com").unwrap(),
            Uri::from_str("sip:bob@192.0.2.4").unwrap()
        );
    }

    #[test]
    fn test_uris_intransitivity() {
        assert_eq!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=on").unwrap()
        );
        assert_eq!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=off").unwrap()
        );
        assert_ne!(
            Uri::from_str("sip:carol@chicago.com;security=on").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=off").unwrap()
        );
    }
}
