//! SIP response types.
//!
//! TODO

use bytes::Bytes;
use itertools::join;
use nom::error::convert_error;
use std::str::from_utf8;

use crate::Reason;
use crate::Version;
use crate::{Error, Header};

/// Representation of a SIP response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    reason: Reason,
    version: Version,
    headers: Vec<Header>,
    body: Bytes,
}

impl Response {
    /// Get a reference to the associated `Reason`.
    #[inline]
    pub fn reason(&self) -> &Reason {
        &self.reason
    }

    /// Get a reference to the associated SIP `Version`.
    #[inline]
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Get a reference to the headers contained in the response.
    pub fn headers(&self) -> &Vec<Header> {
        &self.headers
    }

    /// Get a reference to the associated body.
    #[inline]
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    pub(crate) fn set_body(&mut self, body: &[u8]) {
        self.body = Bytes::copy_from_slice(body);
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}\r\n{}\r\n{}",
            self.version(),
            self.reason(),
            join(self.headers(), "\r\n"),
            match from_utf8(self.body()) {
                Ok(body) => body.to_string(),
                Err(_) => format!("[binary body of size {}]", self.body().len()),
            }
        )
    }
}

impl TryFrom<&str> for Response {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::response(value) {
            Ok((rest, response)) => {
                if !rest.is_empty() {
                    Err(Error::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(response)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(Error::InvalidResponse(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidResponse(format!(
                "Incomplete response `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use super::*;
    use crate::headers::header::parser::message_header;
    use crate::{
        common::{reason::parser::reason, version::parser::sip_version},
        parser::{sp, ParserResult},
    };
    use nom::{
        character::complete::crlf,
        combinator::map,
        error::context,
        multi::many0,
        sequence::{terminated, tuple},
    };

    fn status_line(input: &str) -> ParserResult<&str, (Version, Reason)> {
        context(
            "status_line",
            map(tuple((sip_version, sp, reason)), |(version, _, reason)| {
                (version, reason)
            }),
        )(input)
    }

    pub(crate) fn response(input: &str) -> ParserResult<&str, Response> {
        context(
            "response",
            map(
                tuple((
                    terminated(status_line, crlf),
                    many0(terminated(message_header, crlf)),
                    crlf,
                )),
                |((version, reason), headers, _)| Response {
                    version,
                    reason,
                    headers,
                    body: Bytes::copy_from_slice(&[]),
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
    fn test_valid_response() {
        let ok = Response::try_from("SIP/2.0 200 OK\r\n\r\n");
        assert_ok!(&ok);
        let ok = ok.unwrap();
        assert_eq!(ok.version(), Version::SIP_2);
        assert_eq!(ok.reason(), Reason::OK);
        assert_eq!(ok.headers().len(), 0);

        let not_found = Response::try_from("SIP/2.0 404 Not Found\r\n\r\n");
        assert_ok!(&not_found);
        let not_found = not_found.unwrap();
        assert_eq!(not_found.version(), Version::SIP_2);
        assert_eq!(not_found.reason(), Reason::NOT_FOUND);
        assert_eq!(not_found.reason().to_string(), "404 Not Found");
        assert_eq!(not_found.headers().len(), 0);

        // let with_body = Response::from_bytes(b"SIP/2.0 200 OK\r\n\r\nHello world!");
        // assert_ok!(&with_body);
        // let with_body = with_body.unwrap();
        // assert_eq!(with_body.version(), Version::SIP_2);
        // assert_eq!(with_body.reason(), Reason::OK);
        // assert_eq!(with_body.body(), &Bytes::from_static(b"Hello world!"));

        let unknown_status = Response::try_from("SIP/2.0 999 Mon Status 😁\r\n\r\n");
        assert_ok!(&unknown_status);
        let unknown_status = unknown_status.unwrap();
        assert_eq!(unknown_status.version(), Version::SIP_2);
        assert_eq!(unknown_status.reason(), 999);
        assert_eq!(unknown_status.reason().phrase(), "Mon Status 😁");
        assert_eq!(unknown_status.headers().len(), 0);
    }

    #[test]
    fn test_invalid_response() {
        assert_err!(Response::try_from("Hello world!"));
        assert_err!(Response::try_from("SIP/1.0 200 OK\r\n\r\n"));
    }
}
