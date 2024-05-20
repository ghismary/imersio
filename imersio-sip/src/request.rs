//! SIP request
//!
//! TODO

use bytes::Bytes;

use crate::Error;
use crate::Method;
use crate::Uri;
use crate::Version;

/// Representation of a SIP request.
#[derive(Clone, Debug)]
pub struct Request {
    head: Parts,
    body: Bytes,
}

/// Parts of a SIP `Request`
///
/// The SIP request head consists of a method, uri and version (contained in
/// the Request-Line), and a set of headers.
#[derive(Clone, Debug, Default)]
pub struct Parts {
    /// The request's method.
    pub method: Method,

    /// The request's URI.
    pub uri: Uri,

    /// The request's version.
    pub version: Version,
    // TODO: pub headers: HeaderMap<HeaderValue>,
}

/// A SIP request builder
///
/// This type can be used to construct an instance of `Request` through a
/// builder-like pattern.
#[derive(Debug)]
pub struct Builder {
    inner: Result<Parts, Error>,
}

impl Request {
    /// Create a new `Builder` that will be used to build a `Request`.
    #[inline]
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Try to create a `Request` from a slice of bytes.
    #[inline]
    pub fn from_bytes(input: &[u8]) -> Result<Request, Error> {
        parse(input)
    }

    /// Create a new blank `Request` with the body
    ///
    /// The parts of this request will be set to their default, e.g. the
    /// INVITE method, no headers...
    #[inline]
    pub fn new(body: Bytes) -> Self {
        Request {
            head: Parts::new(),
            body,
        }
    }

    /// Create a new `Builder` initialized with an ACK method and the given URI.
    pub fn ack() -> Builder {
        Builder::new().method(Method::ACK)
    }

    /// Get a reference to the associated SIP method.
    #[inline]
    pub fn method(&self) -> &Method {
        &self.head.method
    }

    /// Get a mutable reference to the associated SIP method.
    #[inline]
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.head.method
    }

    /// Get a reference to the associated SIP URI.
    #[inline]
    pub fn uri(&self) -> &Uri {
        &self.head.uri
    }

    /// Get a mutable reference to the associated SIP URI.
    #[inline]
    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.head.uri
    }

    /// Get the associated SIP version.
    #[inline]
    pub fn version(&self) -> Version {
        self.head.version
    }

    /// Get a mutable reference to the associated SIP version.
    #[inline]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.head.version
    }

    /// Get a reference to the associated body.
    #[inline]
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    /// Get a mutable reference to the associated body.
    #[inline]
    pub fn body_mut(&mut self) -> &mut Bytes {
        &mut self.body
    }
}

impl Default for Request {
    fn default() -> Self {
        Request::new(Bytes::default())
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
    /// Create a new default instance of `Builder` to build a `Request`.
    #[inline]
    pub fn new() -> Self {
        Builder::default()
    }

    /// Set the method for this SIP request.
    ///
    /// By default this is `INVITE`.
    pub fn method<T>(self, method: T) -> Self
    where
        Method: TryFrom<T>,
        <Method as TryFrom<T>>::Error: Into<Error>,
    {
        self.and_then(move |mut head| {
            let method = TryFrom::try_from(method).map_err(Into::into)?;
            head.method = method;
            Ok(head)
        })
    }

    /// Get the method for this SIP request.
    ///
    /// By default this is `INVITE`. If the builder has an error, it returns
    /// None.
    pub fn method_ref(&self) -> Option<&Method> {
        self.inner.as_ref().ok().map(|head| &head.method)
    }

    /// Set the uri for this SIP request.
    pub fn uri(self, uri: Uri) -> Self {
        self.and_then(move |mut head| {
            head.uri = uri;
            Ok(head)
        })
    }

    /// Gte the URI for this SIP request.
    ///
    /// If the builder has an error, it returns None.
    pub fn uri_ref(&self) -> Option<&Uri> {
        self.inner.as_ref().ok().map(|head| &head.uri)
    }

    /// Set the version for this SIP request.
    ///
    /// By default this is SIP/2.0.
    pub fn version(self, version: Version) -> Self {
        self.and_then(move |mut head| {
            head.version = version;
            Ok(head)
        })
    }

    /// Get the version for this SIP request.
    ///
    /// By default this is SIP/2.0.
    pub fn version_ref(&self) -> Option<&Version> {
        self.inner.as_ref().ok().map(|head| &head.version)
    }

    /// Create a `Request`, consuming this builder and using the provided
    /// `body`.
    ///
    /// # Errors
    ///
    /// This function may return an error if any previously configuration
    /// failed to parse or get converted to the internal representation.
    pub fn body(self, body: Bytes) -> Result<Request, Error> {
        self.inner.map(move |head| Request { head, body })
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
    #[inline]
    fn default() -> Self {
        Builder {
            inner: Ok(Parts::new()),
        }
    }
}

fn parse(input: &[u8]) -> Result<Request, Error> {
    match parser::request(input) {
        Ok((_, request)) => request,
        Err(e) => Err(Error::InvalidRequest(e.to_string())),
    }
}

mod parser {
    use super::*;
    use crate::{
        error::Error,
        method::parser::method,
        parser::{sp, ParserResult},
        uri::parser::uri,
        version::parser::sip_version,
    };
    use nom::{
        character::complete::crlf,
        error::context,
        sequence::{terminated, tuple},
    };

    fn request_line(input: &[u8]) -> ParserResult<&[u8], (Method, Uri, Version)> {
        context(
            "request_line",
            tuple((method, sp, uri, sp, sip_version, crlf)),
        )(input)
        .map(|(rest, (method, _, uri, _, version, _))| (rest, (method, uri, version)))
    }

    pub(super) fn request(input: &[u8]) -> ParserResult<&[u8], Result<Request, Error>> {
        context("request", terminated(request_line, crlf))(input).map(
            |(rest, (method, uri, version))| {
                (
                    &b""[..],
                    Request::builder()
                        .method(method)
                        .uri(uri)
                        .version(version)
                        .body(Bytes::copy_from_slice(rest)),
                )
            },
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_request() {
        let req = Request::from_bytes(b"INVITE sip:alice@atlanta.com SIP/2.0\r\n\r\n");
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.method(), Method::INVITE);
        assert_eq!(req.uri().to_string(), "sip:alice@atlanta.com");
        assert_eq!(req.version(), Version::SIP_2);

        let with_body =
            Request::from_bytes(b"REGISTER sip:alice@gateway.com SIP/2.0\r\n\r\nHello world!");
        assert!(with_body.is_ok());
        let with_body = with_body.unwrap();
        assert_eq!(with_body.method(), Method::REGISTER);
        assert_eq!(with_body.uri().to_string(), "sip:alice@gateway.com");
        assert_eq!(with_body.version(), Version::SIP_2);
        assert_eq!(with_body.body(), &Bytes::from_static(b"Hello world!"));
    }

    #[test]
    fn test_invalid_request() {
        assert!(Request::from_bytes(b"Hello world!").is_err());
        assert!(Request::from_bytes(b"INVITE sip:alice@atlanta.com SIP/1.0\r\n\r\n").is_err());
        assert!(
            Request::from_bytes(b"INVITE sip:alice@atlanta.com@gateway.com SIP/2.0\r\n\r\n")
                .is_err()
        );
    }
}
