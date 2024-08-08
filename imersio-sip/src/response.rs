//! SIP response types.
//!
//! TODO

use bytes::Bytes;

use crate::Error;
use crate::Reason;
use crate::Version;

/// Representation of a SIP response.
#[derive(Clone, Debug)]
pub struct Response {
    head: Parts,
    body: Bytes,
}

/// Parts of a SIP `Response`.
///
/// The SIP response head consists of a reason, a version, and a set of
/// headers.
#[derive(Clone, Debug, Default)]
pub struct Parts {
    /// The response's reason
    pub reason: Reason,

    /// The response's version
    pub version: Version,
}

/// A SIP response builder.
///
/// This type can be used to build instances of `Response` using a
/// builder-like pattern.
#[derive(Debug)]
pub struct Builder {
    inner: Result<Parts, Error>,
}

impl Response {
    /// Create a new `Builder` that will be used to build a `Response`.
    #[inline]
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Create a new blank `Response` with the given body.
    ///
    /// The parts of this response will be set to their default, e.g. the
    /// OK status, no headers...
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let response = Response::new(Bytes::from_static(b"hello world"));
    /// assert_eq!(response.reason(), Reason::OK);
    /// assert_eq!(*response.body(), "hello world");
    /// ```
    #[inline]
    pub fn new(body: Bytes) -> Self {
        Response {
            head: Parts::new(),
            body,
        }
    }

    /// Get a reference to the associated `Reason`.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let response: Response = Response::default();
    /// assert_eq!(response.reason(), Reason::OK);
    /// ```
    #[inline]
    pub fn reason(&self) -> &Reason {
        &self.head.reason
    }

    /// Get a mutable reference to the associated SIP `StatusCode`.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let mut response: Response = Response::default();
    /// *response.reason_mut() = Reason::TRYING;
    /// assert_eq!(response.reason(), Reason::TRYING);
    /// ```
    #[inline]
    pub fn reason_mut(&mut self) -> &mut Reason {
        &mut self.head.reason
    }

    /// Get the associated SIP `Version`.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let response: Response = Response::default();
    /// assert_eq!(response.version(), Version::SIP_2);
    /// ```
    #[inline]
    pub fn version(&self) -> Version {
        self.head.version
    }

    /// Get a mutable reference to the associated SIP `Version`.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let mut response: Response = Response::default();
    /// *response.version_mut() = Version::SIP_2;
    /// assert_eq!(response.version(), Version::SIP_2);
    /// ```
    #[inline]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.head.version
    }

    /// Get a reference to the associated body.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let response: Response = Response::default();
    /// assert!(response.body().is_empty());
    /// ```
    #[inline]
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    /// Get a mutable reference to the associated body.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let mut response: Response = Response::default();
    /// *response.body_mut() = Bytes::from_static(b"hello world");
    /// assert!(!response.body().is_empty());
    /// assert_eq!(response.body(), "hello world");
    /// ```
    #[inline]
    pub fn body_mut(&mut self) -> &mut Bytes {
        &mut self.body
    }
}

impl Default for Response {
    #[inline]
    fn default() -> Self {
        Response::new(Bytes::default())
    }
}

impl TryFrom<&str> for Response {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse(value, vec![].as_slice())
    }
}

impl Parts {
    fn new() -> Self {
        Parts {
            ..Default::default()
        }
    }
}

impl Builder {
    /// Create a new default instance of `Builder` to build a `Response`.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let response = Response::builder()
    ///     .reason(Reason::OK)
    ///     .body(Bytes::new())
    ///     .unwrap();
    /// assert_eq!(response.reason(), Reason::OK);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Builder::default()
    }

    /// Set the SIP reason for this response.
    ///
    /// By default, this is `Reason::ok()`.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let response = Response::builder()
    ///     .reason(Reason::RINGING)
    ///     .body(Bytes::new())
    ///     .unwrap();
    /// assert_eq!(response.reason(), Reason::RINGING);
    /// ```
    pub fn reason<T>(self, reason: T) -> Self
    where
        Reason: TryFrom<T>,
        <Reason as TryFrom<T>>::Error: Into<Error>,
    {
        self.and_then(move |mut head| {
            head.reason = TryFrom::try_from(reason).map_err(Into::into)?;
            Ok(head)
        })
    }

    /// Set the SIP version for this response.
    ///
    /// By default, this is SIP/2.0.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let response = Response::builder()
    ///     .reason(Reason::NOT_FOUND)
    ///     .version(Version::SIP_2)
    ///     .body(Bytes::new())
    ///     .unwrap();
    /// assert_eq!(response.version(), Version::SIP_2);
    /// ```
    pub fn version(self, version: Version) -> Self {
        self.and_then(move |mut head| {
            head.version = version;
            Ok(head)
        })
    }

    /// Create a `Response`, consuming this builder and using the provided
    /// `body`.
    ///
    /// # Errors
    ///
    /// This function may return an error if any previously configuration
    /// failed to parse or get converted to the internal representation.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::*;
    /// let response = Response::builder()
    ///     .body(Bytes::new());
    /// assert!(response.is_ok());
    /// ```
    pub fn body(self, body: Bytes) -> Result<Response, Error> {
        self.inner.map(move |head| Response { head, body })
    }

    fn and_then<F>(self, func: F) -> Self
    where
        F: FnOnce(Parts) -> Result<Parts, Error>,
    {
        Builder {
            inner: self.inner.and_then(func),
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            inner: Ok(Parts::new()),
        }
    }
}

fn parse(input: &str, body: &[u8]) -> Result<Response, Error> {
    match parser::response(input, body) {
        Ok((rest, result)) => {
            if rest.is_empty() {
                result
            } else {
                Err(Error::RemainingUnparsedData(rest.to_string()))
            }
        }
        Err(e) => Err(Error::InvalidRequest(e.to_string())),
    }
}

mod parser {
    use super::*;
    use crate::{
        common::{reason::parser::reason, version::parser::sip_version},
        parser::{sp, ParserResult},
    };
    use nom::{
        character::complete::crlf,
        combinator::map,
        error::context,
        sequence::{terminated, tuple},
    };

    fn status_line(input: &str) -> ParserResult<&str, (Version, Reason)> {
        context(
            "status_line",
            map(
                tuple((sip_version, sp, reason, crlf)),
                |(version, _, reason, _)| (version, reason),
            ),
        )(input)
    }

    pub(super) fn response<'input>(
        input: &'input str,
        body: &[u8],
    ) -> ParserResult<&'input str, Result<Response, Error>> {
        context(
            "response",
            map(terminated(status_line, crlf), |(version, reason)| {
                Response::builder()
                    .version(version)
                    .reason(reason)
                    .body(Bytes::copy_from_slice(body))
            }),
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

        let not_found = Response::try_from("SIP/2.0 404 Not Found\r\n\r\n");
        assert_ok!(&not_found);
        let not_found = not_found.unwrap();
        assert_eq!(not_found.version(), Version::SIP_2);
        assert_eq!(not_found.reason(), Reason::NOT_FOUND);
        assert_eq!(not_found.reason().to_string(), "404 Not Found");

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
    }

    #[test]
    fn test_invalid_response() {
        assert_err!(Response::try_from("Hello world!"));
        assert_err!(Response::try_from("SIP/1.0 200 OK\r\n\r\n"));
    }
}
