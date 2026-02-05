//! TODO

use nom_language::error::convert_error;
use std::borrow::Cow;
use std::str;

use crate::{SipError, StatusCode};

/// A SIP response reason, the combination of the `StatusCode` and the reason
/// phrase.
#[derive(Clone, Debug, Eq)]
pub struct Reason {
    status: StatusCode,
    phrase: Cow<'static, str>,
}

impl Reason {
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

    /// Get the reason phrase.
    ///
    /// # Example
    ///
    /// ```
    /// let reason = imersio_sip::Reason::RINGING;
    /// assert_eq!(reason.phrase(), "Ringing");
    /// ```
    pub fn phrase(&self) -> &str {
        &self.phrase
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

impl TryFrom<&str> for Reason {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::reason(value) {
            Ok((rest, reason)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(reason)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidReason(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidReason(format!(
                "Incomplete reason `{}`",
                value
            ))),
        }
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

macro_rules! reasons {
    (
        $(
            $(#[$docs:meta])*
            ($code:expr, $konst:ident, $phrase:expr),
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
    (100, TRYING, "Trying"),
    /// 180 Ringing
    /// [[RFC3261, Section 21.1.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.1.2)]
    (180, RINGING, "Ringing"),
    /// 181 Call Is Being Forwarded
    /// [[RFC3261, Section 21.1.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.1.3)]
    (181, CALL_IS_BEING_FORWARDED, "Call Is Being Forwarded"),
    /// 182 Queued
    /// [[RFC3261, Section 21.1.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.1.4)]
    (182, QUEUED, "Queued"),
    /// 183 Session Progress
    /// [[RFC3261, Section 21.1.5](https://datatracker.ietf.org/doc/html/rfc3261#section-21.1.5)]
    (183, SESSION_PROGRESS, "Session Progress"),

    /// 200 OK
    /// [[RFC3261, Section 21.2.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.2.1)]
    (200, OK, "OK"),

    /// 300 Multiple Choices
    /// [[RFC3261, Section 21.3.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.1)]
    (300, MULTIPLE_CHOICES, "Multiple Choices"),
    /// 301 Moved Permanently
    /// [[RFC3261, Section 21.3.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.2)]
    (301, MOVED_PERMANENTLY, "Moved Permanently"),
    /// 302 Moved Temporarily
    /// [[RFC3261, Section 21.3.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.3)]
    (302, MOVED_TEMPORARILY, "Moved Temporarily"),
    /// 305 Use Proxy
    /// [[RFC3261, Section 21.3.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.4)]
    (305, USE_PROXY, "Use Proxy"),
    /// 380 Alternative Service
    /// [[RFC3261, Section 21.3.5](https://datatracker.ietf.org/doc/html/rfc3261#section-21.3.5)]
    (380, ALTERNATE_SERVICE, "Alternative Service"),

    /// 400 Bad Request
    /// [[RFC3261, Section 21.4.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.1)]
    (400, BAD_REQUEST, "Bad Request"),
    /// 401 Unauthorized
    /// [[RFC3261, Section 21.4.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.2)]
    (401, UNAUTHORIZED, "Unauthorized"),
    /// 402 Payment Required
    /// [[RFC3261, Section 21.4.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.3)]
    (402, PAYMENT_REQUIRED, "Payment Required"),
    /// 403 Forbidden
    /// [[RFC3261, Section 21.4.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.4)]
    (403, FORBIDDEN, "Forbidden"),
    /// 404 Not Found
    /// [[RFC3261, Section 21.4.5](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.5)]
    (404, NOT_FOUND, "Not Found"),
    /// 405 Method Not Allowed
    /// [[RFC3261, Section 21.4.6](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.6)]
    (405, METHOD_NOT_ALLOWED, "Method Not Allowed"),
    /// 406 Not Acceptable
    /// [[RFC3261, Section 21.4.7](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.7)]
    (406, NOT_ACCEPTABLE, "Not Acceptable"),
    /// 407 Proxy Authentication Required
    /// [[RFC3261, Section 21.4.8](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.8)]
    (407, PROXY_AUTHENTICATION_REQUIRED, "Proxy Authentication Required"),
    /// 408 Request Timeout
    /// [[RFC3261, Section 21.4.9](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.9)]
    (408, REQUEST_TIMEOUT, "Request Timeout"),
    /// 410 Gone
    /// [[RFC3261, Section 21.4.10](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.10)]
    (410, GONE, "Gone"),
    /// 413 Request Entity Too Large
    /// [[RFC3261, Section 21.4.11](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.11)]
    (413, REQUEST_ENTITY_TOO_LARGE, "Request Entity Too Large"),
    /// 414 Request-URI Too Long
    /// [[RFC3261, Section 21.4.12](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.12)]
    (414, REQUEST_URI_TOO_LONG, "Request-URI Too Long"),
    /// 415 Unsupported Media Type
    /// [[RFC3261, Section 21.4.13](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.13)]
    (415, UNSUPPORTED_MEDIA_TYPE, "Unsupported Media Type"),
    /// 416 Unsupported URI Scheme
    /// [[RFC3261, Section 21.4.14](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.14)]
    (416, UNSUPPORTED_URI_SCHEME, "Unsupported URI Scheme"),
    /// 420 Bad Extension
    /// [[RFC3261, Section 21.4.15](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.15)]
    (420, BAD_EXTENSION, "Bad Extension"),
    /// 421 Extension Required
    /// [[RFC3261, Section 21.4.16](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.16)]
    (421, EXTENSION_REQUIRED, "Extension Required"),
    /// 423 Interval Too Brief
    /// [[RFC3261, Section 21.4.17](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.17)]
    (423, INTERVAL_TOO_BRIEF, "Interval Too Brief"),
    /// 480 Temporarily Unavailable
    /// [[RFC3261, Section 21.4.18](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.18)]
    (480, TEMPORARILY_UNAVAILABLE, "Temporarily Unavailable"),
    /// 481 Call/Transaction Does Not Exist
    /// [[RFC3261, Section 21.4.19](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.19)]
    (481, CALL_TRANSACTION_DOES_NOT_EXIST, "Call/Transaction Does Not Exist"),
    /// 482 Loop Detected
    /// [[RFC3261, Section 21.4.20](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.20)]
    (482, LOOP_DETECTED, "Loop Detected"),
    /// 483 Too Many Hops
    /// [[RFC3261, Section 21.4.21](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.21)]
    (483, TOO_MANY_HOPS, "Too Many Hops"),
    /// 484 Address Incomplete
    /// [[RFC3261, Section 21.4.22](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.22)]
    (484, ADDRESS_INCOMPLETE, "Address Incomplete"),
    /// 485 Ambiguous
    /// [[RFC3261, Section 21.4.23](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.23)]
    (485, AMBIGUOUS, "Ambiguous"),
    /// 486 Busy Here
    /// [[RFC3261, Section 21.4.24](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.24)]
    (486, BUSY_HERE, "Busy Here"),
    /// 487 Request Terminated
    /// [[RFC3261, Section 21.4.25](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.25)]
    (487, REQUEST_TERMINATED, "Request Terminated"),
    /// 488 Not Acceptable Here
    /// [[RFC3261, Section 21.4.26](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.26)]
    (488, NOT_ACCEPTABLE_HERE, "Not Acceptable Here"),
    /// 491 Request Pending
    /// [[RFC3261, Section 21.4.27](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.27)]
    (491, REQUEST_PENDING, "Request Pending"),
    /// 493 Undecipherable
    /// [[RFC3261, Section 21.4.28](https://datatracker.ietf.org/doc/html/rfc3261#section-21.4.28)]
    (493, UNDECIPHERABLE, "Undecipherable"),

    /// 500 Server Internal Error
    /// [[RFC3261, Section 21.5.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.1)]
    (500, SERVER_INTERNAL_ERROR, "Server Internal Error"),
    /// 501 Not Implemented
    /// [[RFC3261, Section 21.5.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.2)]
    (501, NOT_IMPLEMENTED, "Not Implemented"),
    /// 502 Bad Gateway
    /// [[RFC3261, Section 21.5.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.3)]
    (502, BAD_GATEWAY, "Bad Gateway"),
    /// 503 Service Unavailable
    /// [[RFC3261, Section 21.5.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.4)]
    (503, SERVICE_UNAVAILABLE, "Service Unavailable"),
    /// 504 Server Time-out
    /// [[RFC3261, Section 21.5.5](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.5)]
    (504, SERVER_TIMEOUT, "Server Time-out"),
    /// 505 Version Not Supported
    /// [[RFC3261, Section 21.5.6](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.6)]
    (505, VERSION_NOT_SUPPORTED, "Version Not Supported"),
    /// 513 Message Too Large
    /// [[RFC3261, Section 21.5.7](https://datatracker.ietf.org/doc/html/rfc3261#section-21.5.7)]
    (513, MESSAGE_TOO_LARGE, "Message Too Large"),

    /// 600 Busy Everywhere
    /// [[RFC3261, Section 21.6.1](https://datatracker.ietf.org/doc/html/rfc3261#section-21.6.1)]
    (600, BUSY_EVERYWHERE, "Busy Everywhere"),
    /// 603 Decline
    /// [[RFC3261, Section 21.6.2](https://datatracker.ietf.org/doc/html/rfc3261#section-21.6.2)]
    (603, DECLINE, "Decline"),
    /// 604 Does Not Exist Anywhere
    /// [[RFC3261, Section 21.6.3](https://datatracker.ietf.org/doc/html/rfc3261#section-21.6.3)]
    (604, DOES_NOT_EXIST_ANYWHERE, "Does Not Exist Anywhere"),
    /// 606 Not Acceptable
    /// [[RFC3261, Section 21.6.4](https://datatracker.ietf.org/doc/html/rfc3261#section-21.6.4)]
    (606, NOT_ACCEPTABLE_GLOBAL, "Not Acceptable"),
}

pub(crate) mod parser {
    use nom::{
        Parser,
        branch::alt,
        combinator::{map, value},
        error::context,
        multi::many0,
        sequence::separated_pair,
    };

    use super::*;
    use crate::{
        common::status_code::parser::status_code,
        parser::{ParserResult, escaped, reserved, sp, tab, unreserved, utf8_nonascii},
    };

    fn reason_phrase(input: &str) -> ParserResult<&str, String> {
        context(
            "reason_phrase",
            map(
                many0(alt((
                    reserved,
                    unreserved,
                    escaped,
                    utf8_nonascii,
                    value(' ', sp),
                    value('\t', tab),
                ))),
                |chars| chars.iter().collect(),
            ),
        )
        .parse(input)
    }

    pub(crate) fn reason(input: &str) -> ParserResult<&str, Reason> {
        context(
            "reason",
            map(
                separated_pair(status_code, sp, reason_phrase),
                |(status, phrase)| Reason {
                    status,
                    phrase: Cow::Owned(phrase),
                },
            ),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

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

        let custom_ringing_reason = Reason::try_from("180 Sonne").unwrap();
        assert_eq!(Reason::RINGING, custom_ringing_reason);
    }

    #[test]
    fn test_valid_reason() {
        assert_ok!(Reason::try_from("200 OK"));
        assert_ok!(Reason::try_from("200 Bon"));
        assert_ok!(Reason::try_from("404 Pas Trouvé"));
    }

    #[test]
    fn test_invalid_reason_empty() {
        assert_err!(Reason::try_from(""));
    }

    #[test]
    fn test_invalid_reason_no_status_code() {
        assert_err!(Reason::try_from("Hello world!"));
    }

    #[test]
    fn test_invalid_reason_invalid_status_code() {
        assert_err!(Reason::try_from("4040 Not Found"));
    }
}
