//! SIP request
//!
//! TODO

use itertools::join;
use nom::error::convert_error;
use std::str::from_utf8;

use crate::Method;
use crate::Uri;
use crate::Version;
use crate::{Header, SipError};

/// Representation of a SIP request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    method: Method,
    uri: Uri,
    version: Version,
    headers: Vec<Header>,
    body: Vec<u8>,
}

impl Request {
    /// Get a reference to the associated SIP method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get a reference to the associated SIP URI.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Get a reference to the associated SIP version.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Get a reference to the list of headers of the SIP request.
    pub fn headers(&self) -> &Vec<Header> {
        &self.headers
    }

    /// Get a reference to the associated body.
    #[inline]
    pub fn body(&self) -> &[u8] {
        self.body.as_slice()
    }

    pub(crate) fn set_body(&mut self, body: &[u8]) {
        self.body = body.to_vec();
    }
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}\r\n{}\r\n{}",
            self.method(),
            self.uri(),
            self.version(),
            join(self.headers(), "\r\n"),
            match from_utf8(self.body()) {
                Ok(body) => body.to_string(),
                Err(_) => format!("[binary body of size {}]", self.body().len()),
            }
        )
    }
}

impl TryFrom<&str> for Request {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::request(value) {
            Ok((rest, request)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(request)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidRequest(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidRequest(format!(
                "Incomplete request `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use super::*;
    use crate::headers::header::parser::message_header;
    use crate::uris::uri::parser::request_uri;
    use crate::{
        common::{method::parser::method, version::parser::sip_version},
        parser::{sp, ParserResult},
    };
    use nom::{
        character::complete::crlf,
        combinator::map,
        error::context,
        multi::many0,
        sequence::{terminated, tuple},
    };

    fn request_line(input: &str) -> ParserResult<&str, (Method, Uri, Version)> {
        context(
            "request_line",
            map(
                tuple((method, sp, request_uri, sp, sip_version)),
                |(method, _, uri, _, version)| (method, uri, version),
            ),
        )(input)
    }

    pub(crate) fn request(input: &str) -> ParserResult<&str, Request> {
        context(
            "request",
            map(
                tuple((
                    terminated(request_line, crlf),
                    many0(terminated(message_header, crlf)),
                    crlf,
                )),
                |((method, uri, version), headers, _)| Request {
                    method,
                    uri,
                    version,
                    headers,
                    body: vec![],
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_valid_request() {
        let req = Request::try_from("INVITE sip:alice@atlanta.com SIP/2.0\r\n\r\n");
        assert_ok!(&req);
        let req = req.unwrap();
        assert_eq!(req.method(), &Method::Invite);
        assert_eq!(req.uri().to_string(), "sip:alice@atlanta.com");
        assert_eq!(req.version(), &Version::Sip2);
        assert_eq!(req.headers().len(), 0);
    }

    #[test]
    fn test_invalid_request() {
        assert_err!(Request::try_from("Hello world!"));
        assert_err!(Request::try_from(
            "INVITE sip:alice@atlanta.com SIP/1.0\r\n\r\n"
        ));
        assert_err!(Request::try_from(
            "INVITE sip:alice@atlanta.com@gateway.com SIP/2.0\r\n\r\n"
        ));
    }
}
