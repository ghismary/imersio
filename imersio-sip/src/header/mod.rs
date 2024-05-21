//! TODO

mod accept_encoding_header;
mod accept_header;
mod accept_language_header;
mod alert_info_header;
mod allow_header;
mod authentication_info_header;
mod authorization_header;
mod call_id_header;
mod call_info_header;
mod parser;

use std::str::FromStr;

use accept_encoding_header::AcceptEncodingHeader;
use accept_header::AcceptHeader;
use accept_language_header::AcceptLanguageHeader;
use alert_info_header::AlertInfoHeader;
use allow_header::AllowHeader;
use authentication_info_header::AuthenticationInfoHeader;
use authorization_header::AuthorizationHeader;
use call_id_header::CallIdHeader;
use call_info_header::CallInfoHeader;

use crate::Error;

/// Representation of a SIP message header.
#[derive(Clone, Debug)]
pub enum Header {
    /// An Accept message header.
    Accept(AcceptHeader),
    /// An Accept-Encoding message header.
    AcceptEncoding(AcceptEncodingHeader),
    /// An Accept-Language message header.
    AcceptLanguage(AcceptLanguageHeader),
    /// An Alert-Info message header.
    AlertInfo(AlertInfoHeader),
    /// An Allow message header.
    Allow(AllowHeader),
    /// An Authentication-Info header.
    AuthenticationInfo(AuthenticationInfoHeader),
    /// An Authorization header.
    Authorization(AuthorizationHeader),
    /// A Call-ID header.
    CallId(CallIdHeader),
    /// A Call-Info header.
    CallInfo(CallInfoHeader),
}

impl Header {
    /// Try to create a `Header` from a slice of bytes.
    #[inline]
    pub fn from_bytes(input: &[u8]) -> Result<Header, Error> {
        parse(input)
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Header::Accept(header) => header.to_string(),
                Header::AcceptEncoding(header) => header.to_string(),
                Header::AcceptLanguage(header) => header.to_string(),
                Header::AlertInfo(header) => header.to_string(),
                Header::Allow(header) => header.to_string(),
                Header::AuthenticationInfo(header) => header.to_string(),
                Header::Authorization(header) => header.to_string(),
                Header::CallId(header) => header.to_string(),
                Header::CallInfo(header) => header.to_string(),
            }
        )
    }
}

impl FromStr for Header {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Header::from_bytes(s.as_bytes())
    }
}

fn parse(input: &[u8]) -> Result<Header, Error> {
    match parser::message_header(input) {
        Ok((rest, uri)) => {
            if !rest.is_empty() {
                Err(Error::RemainingUnparsedData)
            } else {
                Ok(uri)
            }
        }
        Err(e) => Err(Error::InvalidMessageHeader(e.to_string())),
    }
}
