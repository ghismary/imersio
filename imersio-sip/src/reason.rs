//! TODO

use std::borrow::Cow;
use std::{str, str::FromStr};

use crate::error::Error;

/// A SIP response reason, the combination of the `StatusCode` and the reason
/// phrase.
#[derive(Clone, Debug)]
pub struct Reason {
    status: StatusCode,
    phrase: Cow<'static, str>,
}

impl Reason {
    /// Try to create a `Reason` from a slice of bytes.
    #[inline]
    pub fn from_bytes(input: &[u8]) -> Result<Reason, Error> {
        parse_reason(input)
    }

    /// Get a reference to the `StatusCode` of the reason.
    ///
    /// # Example
    ///
    /// ```
    /// let reason = imersio_sip::Reason::EXTENSION_REQUIRED;
    /// assert_eq!(reason.status(), imersio_sip::StatusCode::EXTENSION_REQUIRED);
    /// ```
    pub fn status(&self) -> &StatusCode {
        &self.status
    }

    /// Get the reason phrase as a String.
    ///
    /// # Example
    ///
    /// ```
    /// let reason = imersio_sip::Reason::RINGING;
    /// assert_eq!(reason.phrase(), "Ringing");
    /// ```
    pub fn phrase(&self) -> String {
        self.phrase.to_string()
    }

    /// Check if the status code is provisional (between 100 and 199).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::RINGING.is_provisional());
    /// assert!(!imersio_sip::StatusCode::REQUEST_TIMEOUT.is_provisional());
    /// ```
    #[inline]
    pub fn is_provisional(&self) -> bool {
        self.status.is_provisional()
    }

    /// Check if the status code is final (between 200 and 699).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::USE_PROXY.is_final());
    /// assert!(!imersio_sip::StatusCode::TRYING.is_final());
    /// ```
    #[inline]
    pub fn is_final(&self) -> bool {
        self.status.is_final()
    }

    /// Check if the status code is a success (between 200 and 299).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::OK.is_success());
    /// assert!(!imersio_sip::StatusCode::SERVER_INTERNAL_ERROR.is_success());
    /// ```
    #[inline]
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Check if the status code is a redirection (between 300 and 399).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::MOVED_TEMPORARILY.is_redirection());
    /// assert!(!imersio_sip::StatusCode::BUSY_EVERYWHERE.is_redirection());
    /// ```
    #[inline]
    pub fn is_redirection(&self) -> bool {
        self.status.is_redirection()
    }

    /// Check if the status code is a request failure (between 400 and 499).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::NOT_FOUND.is_request_failure());
    /// assert!(!imersio_sip::StatusCode::ALTERNATE_SERVICE.is_request_failure());
    /// ```
    #[inline]
    pub fn is_request_failure(&self) -> bool {
        self.status.is_request_failure()
    }

    /// Check if the status code is a server failure (between 500 and 599).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::NOT_IMPLEMENTED.is_server_failure());
    /// assert!(!imersio_sip::StatusCode::QUEUED.is_server_failure());
    /// ```
    #[inline]
    pub fn is_server_failure(&self) -> bool {
        self.status.is_server_failure()
    }

    /// Check if the status code is a global failure (between 600 and 699).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::DOES_NOT_EXIST_ANYWHERE.is_global_failure());
    /// assert!(!imersio_sip::StatusCode::LOOP_DETECTED.is_global_failure());
    /// ```
    #[inline]
    pub fn is_global_failure(&self) -> bool {
        self.status.is_global_failure()
    }
}

impl Default for Reason {
    #[inline]
    fn default() -> Self {
        Reason::OK
    }
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.status.as_str(), self.phrase)
    }
}

impl FromStr for Reason {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Reason::from_bytes(s.as_bytes())
    }
}

impl PartialEq<Reason> for Reason {
    fn eq(&self, other: &Reason) -> bool {
        self.status == other.status
    }
}

impl PartialEq<u16> for Reason {
    fn eq(&self, other: &u16) -> bool {
        self.status.code() == *other
    }
}

impl PartialEq<Reason> for u16 {
    fn eq(&self, other: &Reason) -> bool {
        *self == other.status.code()
    }
}

impl PartialEq<&Reason> for u16 {
    fn eq(&self, other: &&Reason) -> bool {
        *self == other.status.code()
    }
}

impl PartialEq<u16> for &Reason {
    fn eq(&self, other: &u16) -> bool {
        self.status.code() == *other
    }
}

impl PartialEq<&Reason> for Reason {
    fn eq(&self, other: &&Reason) -> bool {
        self.status == other.status
    }
}

impl PartialEq<Reason> for &Reason {
    fn eq(&self, other: &Reason) -> bool {
        self.status == other.status
    }
}

/// A SIP response status code (`Status-Code` in RFC3261).
///
/// Specific constants are provided for known status codes, described in the
/// [section 21 of RFC3261](https://datatracker.ietf.org/doc/html/rfc3261#section-21).
///
/// Status code values in the range 100-999 (inclusive) are supported by this
/// type. Values in the range 100-699 are semantically classified by the most
/// significant digit, either as provisional, success, redirection, request
/// failure, server failure or global failure. Values above 699 are
/// unclassified but allowed for compatibility, though their use is
/// discouraged. These would probably be interpreted as protocol errors by the
/// application.
///
/// # Examples
///
/// ```
/// use imersio_sip::StatusCode;
///
/// assert_eq!(StatusCode::from_u16(200).unwrap(), StatusCode::OK);
/// assert_eq!(StatusCode::NOT_FOUND.code(), 404);
/// assert!(StatusCode::TRYING.is_provisional());
/// assert!(StatusCode::OK.is_final());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusCode(u16);

impl StatusCode {
    /// Try to create a `Reason` from a slice of bytes.
    #[inline]
    pub fn from_bytes(input: &[u8]) -> Result<StatusCode, Error> {
        parse_status_code(input)
    }

    /// Convert a u16 to a status code.
    ///
    /// The function validates the correctness of the supplied u16. It must be
    /// greater or equal to 100 and less than 1000.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::StatusCode;
    ///
    /// let trying = StatusCode::from_u16(100).unwrap();
    /// assert_eq!(trying, StatusCode::TRYING);
    ///
    /// let err1 = StatusCode::from_u16(99);
    /// assert!(err1.is_err());
    ///
    /// let err2 = StatusCode::from_u16(2738);
    /// assert!(err2.is_err());
    /// ```
    pub fn from_u16(src: u16) -> Result<StatusCode, Error> {
        if !(100..1000).contains(&src) {
            return Err(Error::InvalidStatusCode(
                "not between 100 & 1000".to_string(),
            ));
        }

        Ok(Self(src))
    }

    /// Get a &str representation of the `StatusCode`.
    ///
    /// # Example
    ///
    /// ```
    /// let status = imersio_sip::StatusCode::OK;
    /// assert_eq!(status.as_str(), "200");
    /// ```
    pub fn as_str(&self) -> &str {
        let offset = (self.code() - 100) as usize;
        let offset = offset * 3;

        // Invariant: self has checked range [100, 999] and CODE_DIGITS is
        // ASCII-only, of length 900 * 3 = 2700 bytes.
        &CODE_DIGITS[offset..offset + 3]
    }

    /// Get the `u16` corresponding to this `StatusCode`.
    ///
    /// # Note
    ///
    /// The same can be achieved with the `From<StatusCode>` implementation.
    ///
    /// # Example
    ///
    /// ```
    /// let status = imersio_sip::StatusCode::NOT_FOUND;
    /// assert_eq!(status.code(), 404);
    /// ```
    pub fn code(&self) -> u16 {
        self.0
    }

    /// Check if the status code is provisional (between 100 and 199).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::RINGING.is_provisional());
    /// assert!(!imersio_sip::StatusCode::REQUEST_TIMEOUT.is_provisional());
    /// ```
    #[inline]
    pub fn is_provisional(&self) -> bool {
        self.code() >= 100 && self.code() < 200
    }

    /// Check if the status code is final (between 200 and 699).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::USE_PROXY.is_final());
    /// assert!(!imersio_sip::StatusCode::TRYING.is_final());
    /// ```
    #[inline]
    pub fn is_final(&self) -> bool {
        self.code() >= 200 && self.code() < 700
    }

    /// Check if the status code is a success (between 200 and 299).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::OK.is_success());
    /// assert!(!imersio_sip::StatusCode::SERVER_INTERNAL_ERROR.is_success());
    /// ```
    #[inline]
    pub fn is_success(&self) -> bool {
        self.code() >= 200 && self.code() < 300
    }

    /// Check if the status code is a redirection (between 300 and 399).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::MOVED_TEMPORARILY.is_redirection());
    /// assert!(!imersio_sip::StatusCode::BUSY_EVERYWHERE.is_redirection());
    /// ```
    #[inline]
    pub fn is_redirection(&self) -> bool {
        self.code() >= 300 && self.code() < 400
    }

    /// Check if the status code is a request failure (between 400 and 499).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::NOT_FOUND.is_request_failure());
    /// assert!(!imersio_sip::StatusCode::ALTERNATE_SERVICE.is_request_failure());
    /// ```
    #[inline]
    pub fn is_request_failure(&self) -> bool {
        self.code() >= 400 && self.code() < 500
    }

    /// Check if the status code is a server failure (between 500 and 599).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::NOT_IMPLEMENTED.is_server_failure());
    /// assert!(!imersio_sip::StatusCode::QUEUED.is_server_failure());
    /// ```
    #[inline]
    pub fn is_server_failure(&self) -> bool {
        self.code() >= 500 && self.code() < 600
    }

    /// Check if the status code is a global failure (between 600 and 699).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::DOES_NOT_EXIST_ANYWHERE.is_global_failure());
    /// assert!(!imersio_sip::StatusCode::LOOP_DETECTED.is_global_failure());
    /// ```
    #[inline]
    pub fn is_global_failure(&self) -> bool {
        self.code() >= 600 && self.code() < 699
    }
}

impl Default for StatusCode {
    #[inline]
    fn default() -> Self {
        StatusCode::OK
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl FromStr for StatusCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        StatusCode::from_bytes(s.as_bytes())
    }
}

impl From<StatusCode> for u16 {
    #[inline]
    fn from(value: StatusCode) -> Self {
        value.code()
    }
}

impl From<&StatusCode> for StatusCode {
    #[inline]
    fn from(value: &StatusCode) -> Self {
        value.to_owned()
    }
}

impl TryFrom<&[u8]> for StatusCode {
    type Error = Error;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        StatusCode::from_bytes(value)
    }
}

impl TryFrom<&str> for StatusCode {
    type Error = Error;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<u16> for StatusCode {
    type Error = Error;

    #[inline]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        StatusCode::from_u16(value)
    }
}

impl PartialEq<u16> for StatusCode {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.code() == *other
    }
}

impl PartialEq<StatusCode> for u16 {
    #[inline]
    fn eq(&self, other: &StatusCode) -> bool {
        *self == other.code()
    }
}

impl PartialEq<&StatusCode> for StatusCode {
    fn eq(&self, other: &&StatusCode) -> bool {
        self == *other
    }
}

impl PartialEq<StatusCode> for &StatusCode {
    fn eq(&self, other: &StatusCode) -> bool {
        *self == other
    }
}

macro_rules! reasons {
    (
        $(
            $(#[$docs:meta])*
            ($code:expr, $konst:ident, $phrase:expr);
        )+
    ) => {
        impl StatusCode {
            $(
                $(#[$docs])*
                pub const $konst: StatusCode = StatusCode($code);
            )+
        }

        impl Reason {
            $(
                $(#[$docs])*
                pub const $konst: Reason = Reason { status: StatusCode::$konst, phrase: Cow::Borrowed($phrase)};
            )+
        }
    }
}

reasons! {
    /// 100 Trying
    /// [[RFC3261, Section 21.1.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.1.1)]
    (100, TRYING, "Trying");
    /// 180 Ringing
    /// [[RFC3261, Section 21.1.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.1.2)]
    (180, RINGING, "Ringing");
    /// 181 Call Is Being Forwarded
    /// [[RFC3261, Section 21.1.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.1.3)]
    (181, CALL_IS_BEING_FORWARDED, "Call Is Being Forwarded");
    /// 182 Queued
    /// [[RFC3261, Section 21.1.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.1.4)]
    (182, QUEUED, "Queued");
    /// 183 Session Progress
    /// [[RFC3261, Section 21.1.5](https://datatracker.ietf.org/doc/html/rfc3261#section-21.1.5)]
    (183, SESSION_PROGRESS, "Session Progress");

    /// 200 OK
    /// [[RFC3261, Section 21.2.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.2.1)]
    (200, OK, "OK");

    /// 300 Multiple Choices
    /// [[RFC3261, Section 21.3.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.1)]
    (300, MULTIPLE_CHOICES, "Multiple Choices");
    /// 301 Moved Permanently
    /// [[RFC3261, Section 21.3.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.2)]
    (301, MOVED_PERMANENTLY, "Moved Permanently");
    /// 302 Moved Temporarily
    /// [[RFC3261, Section 21.3.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.3)]
    (302, MOVED_TEMPORARILY, "Moved Temporarily");
    /// 305 Use Proxy
    /// [[RFC3261, Section 21.3.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.4)]
    (305, USE_PROXY, "Use Proxy");
    /// 380 Alternative Service
    /// [[RFC3261, Section 21.3.5](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.5)]
    (380, ALTERNATE_SERVICE, "Alternative Service");

    /// 400 Bad Request
    /// [[RFC3261, Section 21.4.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.1)]
    (400, BAD_REQUEST, "Bad Request");
    /// 401 Unauthorized
    /// [[RFC3261, Section 21.4.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.2)]
    (401, UNAUTHORIZED, "Unauthorized");
    /// 402 Payment Required
    /// [[RFC3261, Section 21.4.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.3)]
    (402, PAYMENT_REQUIRED, "Payment Required");
    /// 403 Forbidden
    /// [[RFC3261, Section 21.4.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.4)]
    (403, FORBIDDEN, "Forbidden");
    /// 404 Not Found
    /// [[RFC3261, Section 21.4.5](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.5)]
    (404, NOT_FOUND, "Not Found");
    /// 405 Method Not Allowed
    /// [[RFC3261, Section 21.4.6](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.6)]
    (405, METHOD_NOT_ALLOWED, "Method Not Allowed");
    /// 406 Not Acceptable
    /// [[RFC3261, Section 21.4.7](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.7)]
    (406, NOT_ACCEPTABLE, "Not Acceptable");
    /// 407 Proxy Authentication Required
    /// [[RFC3261, Section 21.4.8](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.8)]
    (407, PROXY_AUTHENTICATION_REQUIRED, "Proxy Authentication Required");
    /// 408 Request Timeout
    /// [[RFC3261, Section 21.4.9](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.9)]
    (408, REQUEST_TIMEOUT, "Request Timeout");
    /// 410 Gone
    /// [[RFC3261, Section 21.4.10](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.10)]
    (410, GONE, "Gone");
    /// 413 Request Entity Too Large
    /// [[RFC3261, Section 21.4.11](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.11)]
    (413, REQUEST_ENTITY_TOO_LARGE, "Request Entity Too Large");
    /// 414 Request-URI Too Long
    /// [[RFC3261, Section 21.4.12](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.12)]
    (414, REQUEST_URI_TOO_LONG, "Request-URI Too Long");
    /// 415 Unsupported Media Type
    /// [[RFC3261, Section 21.4.13](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.13)]
    (415, UNSUPPORTED_MEDIA_TYPE, "Unsupported Media Type");
    /// 416 Unsupported URI Scheme
    /// [[RFC3261, Section 21.4.14](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.14)]
    (416, UNSUPPORTED_URI_SCHEME, "Unsupported URI Scheme");
    /// 420 Bad Extension
    /// [[RFC3261, Section 21.4.15](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.15)]
    (420, BAD_EXTENSION, "Bad Extension");
    /// 421 Extension Required
    /// [[RFC3261, Section 21.4.16](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.16)]
    (421, EXTENSION_REQUIRED, "Extension Required");
    /// 423 Interval Too Brief
    /// [[RFC3261, Section 21.4.17](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.17)]
    (423, INTERVAL_TOO_BRIEF, "Interval Too Brief");
    /// 480 Temporarily Unavailable
    /// [[RFC3261, Section 21.4.18](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.18)]
    (480, TEMPORARILY_UNAVAILABLE, "Temporarily Unavailable");
    /// 481 Call/Transaction Does Not Exist
    /// [[RFC3261, Section 21.4.19](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.19)]
    (481, CALL_TRANSACTION_DOES_NOT_EXIST, "Call/Transaction Does Not Exist");
    /// 482 Loop Detected
    /// [[RFC3261, Section 21.4.20](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.20)]
    (482, LOOP_DETECTED, "Loop Detected");
    /// 483 Too Many Hops
    /// [[RFC3261, Section 21.4.21](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.21)]
    (483, TOO_MANY_HOPS, "Too Many Hops");
    /// 484 Address Incomplete
    /// [[RFC3261, Section 21.4.22](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.22)]
    (484, ADDRESS_INCOMPLETE, "Address Incomplete");
    /// 485 Ambiguous
    /// [[RFC3261, Section 21.4.23](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.23)]
    (485, AMBIGUOUS, "Ambiguous");
    /// 486 Busy Here
    /// [[RFC3261, Section 21.4.24](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.24)]
    (486, BUSY_HERE, "Busy Here");
    /// 487 Request Terminated
    /// [[RFC3261, Section 21.4.25](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.25)]
    (487, REQUEST_TERMINATED, "Request Terminated");
    /// 488 Not Acceptable Here
    /// [[RFC3261, Section 21.4.26](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.26)]
    (488, NOT_ACCEPTABLE_HERE, "Not Acceptable Here");
    /// 491 Request Pending
    /// [[RFC3261, Section 21.4.27](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.27)]
    (491, REQUEST_PENDING, "Request Pending");
    /// 493 Undecipherable
    /// [[RFC3261, Section 21.4.28](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.28)]
    (493, UNDECIPHERABLE, "Undecipherable");

    /// 500 Server Internal Error
    /// [[RFC3261, Section 21.5.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.1)]
    (500, SERVER_INTERNAL_ERROR, "Server Internal Error");
    /// 501 Not Implemented
    /// [[RFC3261, Section 21.5.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.2)]
    (501, NOT_IMPLEMENTED, "Not Implemented");
    /// 502 Bad Gateway
    /// [[RFC3261, Section 21.5.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.3)]
    (502, BAD_GATEWAY, "Bad Gateway");
    /// 503 Service Unavailable
    /// [[RFC3261, Section 21.5.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.4)]
    (503, SERVICE_UNAVAILABLE, "Service Unavailable");
    /// 504 Server Time-out
    /// [[RFC3261, Section 21.5.5](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.5)]
    (504, SERVER_TIMEOUT, "Server Time-out");
    /// 505 Version Not Supported
    /// [[RFC3261, Section 21.5.6](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.6)]
    (505, VERSION_NOT_SUPPORTED, "Version Not Supported");
    /// 513 Message Too Large
    /// [[RFC3261, Section 21.5.7](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.7)]
    (513, MESSAGE_TOO_LARGE, "Message Too Large");

    /// 600 Busy Everywhere
    /// [[RFC3261, Section 21.6.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.6.1)]
    (600, BUSY_EVERYWHERE, "Busy Everywhere");
    /// 603 Decline
    /// [[RFC3261, Section 21.6.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.6.2)]
    (603, DECLINE, "Decline");
    /// 604 Does Not Exist Anywhere
    /// [[RFC3261, Section 21.6.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.6.3)]
    (604, DOES_NOT_EXIST_ANYWHERE, "Does Not Exist Anywhere");
    /// 606 Not Acceptable
    /// [[RFC3261, Section 21.6.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.6.4)]
    (606, NOT_ACCEPTABLE_GLOBAL, "Not Acceptable");
}

// A string of packed 3-ASCII-digit status code values for the supported range
// of [100, 999] (900 codes, 2700 bytes).
const CODE_DIGITS: &str = "\
100101102103104105106107108109110111112113114115116117118119\
120121122123124125126127128129130131132133134135136137138139\
140141142143144145146147148149150151152153154155156157158159\
160161162163164165166167168169170171172173174175176177178179\
180181182183184185186187188189190191192193194195196197198199\
200201202203204205206207208209210211212213214215216217218219\
220221222223224225226227228229230231232233234235236237238239\
240241242243244245246247248249250251252253254255256257258259\
260261262263264265266267268269270271272273274275276277278279\
280281282283284285286287288289290291292293294295296297298299\
300301302303304305306307308309310311312313314315316317318319\
320321322323324325326327328329330331332333334335336337338339\
340341342343344345346347348349350351352353354355356357358359\
360361362363364365366367368369370371372373374375376377378379\
380381382383384385386387388389390391392393394395396397398399\
400401402403404405406407408409410411412413414415416417418419\
420421422423424425426427428429430431432433434435436437438439\
440441442443444445446447448449450451452453454455456457458459\
460461462463464465466467468469470471472473474475476477478479\
480481482483484485486487488489490491492493494495496497498499\
500501502503504505506507508509510511512513514515516517518519\
520521522523524525526527528529530531532533534535536537538539\
540541542543544545546547548549550551552553554555556557558559\
560561562563564565566567568569570571572573574575576577578579\
580581582583584585586587588589590591592593594595596597598599\
600601602603604605606607608609610611612613614615616617618619\
620621622623624625626627628629630631632633634635636637638639\
640641642643644645646647648649650651652653654655656657658659\
660661662663664665666667668669670671672673674675676677678679\
680681682683684685686687688689690691692693694695696697698699\
700701702703704705706707708709710711712713714715716717718719\
720721722723724725726727728729730731732733734735736737738739\
740741742743744745746747748749750751752753754755756757758759\
760761762763764765766767768769770771772773774775776777778779\
780781782783784785786787788789790791792793794795796797798799\
800801802803804805806807808809810811812813814815816817818819\
820821822823824825826827828829830831832833834835836837838839\
840841842843844845846847848849850851852853854855856857858859\
860861862863864865866867868869870871872873874875876877878879\
880881882883884885886887888889890891892893894895896897898899\
900901902903904905906907908909910911912913914915916917918919\
920921922923924925926927928929930931932933934935936937938939\
940941942943944945946947948949950951952953954955956957958959\
960961962963964965966967968969970971972973974975976977978979\
980981982983984985986987988989990991992993994995996997998999";

fn parse_reason(input: &[u8]) -> Result<Reason, Error> {
    match parser::reason(input) {
        Ok((rest, reason)) => {
            if !rest.is_empty() {
                Err(Error::RemainingUnparsedData)
            } else {
                Ok(reason)
            }
        }
        Err(e) => Err(Error::InvalidReason(e.to_string())),
    }
}

fn parse_status_code(input: &[u8]) -> Result<StatusCode, Error> {
    match parser::status_code(input) {
        Ok((rest, method)) => {
            if !rest.is_empty() {
                Err(Error::RemainingUnparsedData)
            } else {
                Ok(method)
            }
        }
        Err(e) => Err(Error::InvalidStatusCode(e.to_string())),
    }
}

pub(crate) mod parser {
    use super::*;
    use crate::parser::{
        digit, escaped, reserved, sp, tab, unreserved, utf8_cont, utf8_nonascii, ParserResult,
    };
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{recognize, value},
        error::context,
        multi::{count, many0},
        sequence::separated_pair,
    };

    #[inline]
    fn informational(input: &[u8]) -> ParserResult<&[u8], StatusCode> {
        alt((
            value(StatusCode::TRYING, tag("100")),
            value(StatusCode::RINGING, tag("180")),
            value(StatusCode::CALL_IS_BEING_FORWARDED, tag("181")),
            value(StatusCode::QUEUED, tag("182")),
            value(StatusCode::SESSION_PROGRESS, tag("183")),
        ))(input)
    }

    #[inline]
    fn success(input: &[u8]) -> ParserResult<&[u8], StatusCode> {
        value(StatusCode::OK, tag("200"))(input)
    }

    #[inline]
    fn redirection(input: &[u8]) -> ParserResult<&[u8], StatusCode> {
        alt((
            value(StatusCode::MULTIPLE_CHOICES, tag("300")),
            value(StatusCode::MOVED_PERMANENTLY, tag("301")),
            value(StatusCode::MOVED_TEMPORARILY, tag("302")),
            value(StatusCode::USE_PROXY, tag("305")),
            value(StatusCode::ALTERNATE_SERVICE, tag("380")),
        ))(input)
    }

    #[inline]
    fn client_error(input: &[u8]) -> ParserResult<&[u8], StatusCode> {
        alt((
            value(StatusCode::BAD_REQUEST, tag("400")),
            value(StatusCode::UNAUTHORIZED, tag("401")),
            value(StatusCode::PAYMENT_REQUIRED, tag("402")),
            value(StatusCode::FORBIDDEN, tag("403")),
            value(StatusCode::NOT_FOUND, tag("404")),
            value(StatusCode::METHOD_NOT_ALLOWED, tag("405")),
            value(StatusCode::NOT_ACCEPTABLE, tag("406")),
            value(StatusCode::PROXY_AUTHENTICATION_REQUIRED, tag("407")),
            value(StatusCode::REQUEST_TIMEOUT, tag("408")),
            value(StatusCode::GONE, tag("410")),
            value(StatusCode::REQUEST_ENTITY_TOO_LARGE, tag("413")),
            value(StatusCode::REQUEST_URI_TOO_LONG, tag("414")),
            value(StatusCode::UNSUPPORTED_MEDIA_TYPE, tag("415")),
            value(StatusCode::UNSUPPORTED_URI_SCHEME, tag("416")),
            value(StatusCode::BAD_EXTENSION, tag("420")),
            value(StatusCode::EXTENSION_REQUIRED, tag("421")),
            value(StatusCode::INTERVAL_TOO_BRIEF, tag("423")),
            alt((
                value(StatusCode::TEMPORARILY_UNAVAILABLE, tag("480")),
                value(StatusCode::CALL_TRANSACTION_DOES_NOT_EXIST, tag("481")),
                value(StatusCode::LOOP_DETECTED, tag("482")),
                value(StatusCode::TOO_MANY_HOPS, tag("483")),
                value(StatusCode::ADDRESS_INCOMPLETE, tag("484")),
                value(StatusCode::AMBIGUOUS, tag("485")),
                value(StatusCode::BUSY_HERE, tag("486")),
                value(StatusCode::REQUEST_TERMINATED, tag("487")),
                value(StatusCode::NOT_ACCEPTABLE_HERE, tag("488")),
                value(StatusCode::REQUEST_PENDING, tag("491")),
                value(StatusCode::UNDECIPHERABLE, tag("493")),
            )),
        ))(input)
    }

    #[inline]
    fn server_error(input: &[u8]) -> ParserResult<&[u8], StatusCode> {
        alt((
            value(StatusCode::SERVER_INTERNAL_ERROR, tag("500")),
            value(StatusCode::NOT_IMPLEMENTED, tag("501")),
            value(StatusCode::BAD_GATEWAY, tag("502")),
            value(StatusCode::SERVICE_UNAVAILABLE, tag("503")),
            value(StatusCode::SERVER_TIMEOUT, tag("504")),
            value(StatusCode::VERSION_NOT_SUPPORTED, tag("505")),
            value(StatusCode::MESSAGE_TOO_LARGE, tag("513")),
        ))(input)
    }

    #[inline]
    fn global_failure(input: &[u8]) -> ParserResult<&[u8], StatusCode> {
        alt((
            value(StatusCode::BUSY_EVERYWHERE, tag("600")),
            value(StatusCode::DECLINE, tag("603")),
            value(StatusCode::DOES_NOT_EXIST_ANYWHERE, tag("604")),
            value(StatusCode::NOT_ACCEPTABLE_GLOBAL, tag("606")),
        ))(input)
    }

    #[inline]
    fn extension_code(input: &[u8]) -> ParserResult<&[u8], StatusCode> {
        recognize(count(digit, 3))(input).map(|(rest, result)| {
            let a = result[0].wrapping_sub(b'0') as u16;
            let b = result[1].wrapping_sub(b'0') as u16;
            let c = result[2].wrapping_sub(b'0') as u16;
            let status = (a * 100) + (b * 10) + c;
            (rest, StatusCode(status))
        })
    }

    pub(super) fn status_code(input: &[u8]) -> ParserResult<&[u8], StatusCode> {
        context(
            "status_code",
            alt((
                informational,
                success,
                redirection,
                client_error,
                server_error,
                global_failure,
                extension_code,
            )),
        )(input)
    }

    fn reason_phrase(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
        context(
            "reason_phrase",
            recognize(many0(alt((
                reserved,
                unreserved,
                escaped,
                utf8_nonascii,
                utf8_cont,
                sp,
                tab,
            )))),
        )(input)
    }

    pub(crate) fn reason(input: &[u8]) -> ParserResult<&[u8], Reason> {
        separated_pair(status_code, sp, reason_phrase)(input).map(|(rest, result)| {
            (
                rest,
                Reason {
                    status: result.0,
                    phrase: Cow::Owned(String::from_utf8_lossy(result.1).into_owned()),
                },
            )
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_invalid_status_code() {
        assert!(StatusCode::from_u16(10).is_err());
        assert!(StatusCode::from_u16(3478).is_err());
        assert!(StatusCode::from_str("bob").is_err());
        assert!(StatusCode::from_str("9273").is_err());
        assert!(StatusCode::from_bytes(b"4629").is_err());
    }

    #[test]
    fn test_status_code_eq() {
        assert_eq!(StatusCode::OK, 200);
        assert_eq!(200, StatusCode::OK);

        assert_eq!(StatusCode::RINGING, StatusCode::from_u16(180).unwrap());
        assert_eq!(StatusCode::from_u16(180).unwrap(), StatusCode::RINGING);
    }

    #[test]
    fn test_invalid_reason() {
        assert!(Reason::from_str("Hello world!").is_err());
        assert!(Reason::from_str("4040 Not Found").is_err());
    }

    #[test]
    fn test_valid_reason() {
        assert!(Reason::from_bytes(b"200 OK").is_ok());
        assert!(Reason::from_bytes(b"200 Bon").is_ok());
        assert!(Reason::from_str("404 Pas Trouvé").is_ok());
    }

    #[test]
    fn test_reason_eq() {
        assert_eq!(Reason::OK, 200);
        assert_eq!(&Reason::OK, 200);

        assert_eq!(200, Reason::OK);
        assert_eq!(200, &Reason::OK);

        assert_eq!(&Reason::OK, Reason::OK);
        assert_eq!(Reason::OK, &Reason::OK);

        assert_eq!(Reason::OK.to_string(), "200 OK");
        assert_eq!(Reason::OK, 200);

        let custom_ringing_reason = Reason::from_str("180 Sonne").unwrap();
        assert_eq!(Reason::RINGING, custom_ringing_reason);
    }
}
