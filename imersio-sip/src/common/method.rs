//! The SIP Request Method
//!
//! This module contains structs and errors for SIP methods. The main type in
//! this module is `Method`. You should not directly use the type from this
//! module but rather the `imersio_sip::Method` type.
//!
//! # Examples
//!
//! ```
//! use imersio_sip::Method;
//!
//! assert_eq!(Method::Invite, Method::try_from("INVITE").unwrap());
//! assert_eq!(Method::Bye.as_str(), "BYE");
//! let method = Method::try_from("CANCEL");
//! assert!(method.is_ok_and(|method| method == Method::Cancel));
//! ```

use crate::common::value_collection::ValueCollection;
use nom_language::error::convert_error;
use std::{hash::Hash, str};

use crate::{SipError, TokenString};

/// Representation of the list of methods from an `AllowHeader`.
///
/// This is usable as an iterator.
pub type Methods = ValueCollection<Method>;

/// The Request Method
///
/// This type also contains constants for the SIP methods defined in
/// [RFC 3261](https://datatracker.ietf.org/doc/html/rfc3261#section-27.4).
///
/// # Example
///
/// ```
/// use imersio_sip::Method;
///
/// assert_eq!(Method::Invite, Method::try_from("INVITE").unwrap());
/// assert_eq!(Method::Register.as_str(), "REGISTER");
/// ```
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Method {
    /// ACK method.
    Ack,
    /// BYE method.
    Bye,
    /// CANCEL method.
    Cancel,
    /// INVITE method.
    #[default]
    Invite,
    /// OPTIONS method.
    Options,
    /// REGISTER method.
    Register,
    /// Any other method.
    Other(TokenString),
}

impl Method {
    pub(crate) fn new(method: TokenString) -> Self {
        let method = method.to_ascii_uppercase();
        match method.as_str() {
            "ACK" => Self::Ack,
            "BYE" => Self::Bye,
            "CANCEL" => Self::Cancel,
            "INVITE" => Self::Invite,
            "OPTIONS" => Self::Options,
            "REGISTER" => Self::Register,
            _ => Self::Other(TokenString::new(method)),
        }
    }

    /// Return a &str representation of the SIP method.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Ack => "ACK",
            Self::Bye => "BYE",
            Self::Cancel => "CANCEL",
            Self::Invite => "INVITE",
            Self::Options => "OPTIONS",
            Self::Register => "REGISTER",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for Method {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for Method {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl PartialEq<Method> for &str {
    #[inline]
    fn eq(&self, other: &Method) -> bool {
        *self == other.as_ref()
    }
}

impl TryFrom<&str> for Method {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::method(value) {
            Ok((rest, method)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(method)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidMethod(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidMethod(format!(
                "Incomplete method `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, value},
        error::context,
        Parser,
    };

    use super::Method;
    use crate::parser::{token, ParserResult};

    #[inline]
    fn ack_method(input: &str) -> ParserResult<&str, Method> {
        value(Method::Ack, tag("ACK")).parse(input)
    }

    #[inline]
    fn bye_method(input: &str) -> ParserResult<&str, Method> {
        value(Method::Bye, tag("BYE")).parse(input)
    }

    #[inline]
    fn cancel_method(input: &str) -> ParserResult<&str, Method> {
        value(Method::Cancel, tag("CANCEL")).parse(input)
    }

    #[inline]
    fn extension_method(input: &str) -> ParserResult<&str, Method> {
        map(token, Method::Other).parse(input)
    }

    #[inline]
    fn invite_method(input: &str) -> ParserResult<&str, Method> {
        value(Method::Invite, tag("INVITE")).parse(input)
    }

    #[inline]
    fn options_method(input: &str) -> ParserResult<&str, Method> {
        value(Method::Options, tag("OPTIONS")).parse(input)
    }

    #[inline]
    fn register_method(input: &str) -> ParserResult<&str, Method> {
        value(Method::Register, tag("REGISTER")).parse(input)
    }

    pub(crate) fn method(input: &str) -> ParserResult<&str, Method> {
        context(
            "method",
            alt((
                invite_method,
                ack_method,
                options_method,
                bye_method,
                cancel_method,
                register_method,
                extension_method,
            )),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::assert_err;

    #[test]
    fn test_method_eq() {
        assert_eq!(Method::Invite, "INVITE");
        assert_eq!("INVITE", Method::Invite);
    }

    #[test]
    fn test_valid_method() {
        assert!(Method::try_from("INVITE").is_ok_and(|method| method == Method::Invite));
        assert!(Method::try_from("CANCEL").is_ok_and(|method| method == Method::Cancel));
        assert_eq!(Method::Invite.as_str(), "INVITE");
    }

    #[test]
    fn test_invalid_method_empty() {
        assert_err!(Method::try_from(""));
    }

    #[test]
    fn test_invalid_method_with_invalid_character() {
        assert_err!(Method::try_from("\n"));
    }

    #[test]
    fn test_invalid_method_with_remaining_data() {
        assert!(Method::try_from("INVITE anything")
            .is_err_and(|e| e == SipError::RemainingUnparsedData(" anything".to_string())));
    }

    #[test]
    fn test_extension_method() {
        assert_eq!(Method::try_from("EXTENSION").unwrap(), "EXTENSION");
        assert_eq!(Method::try_from("ex-Tension.").unwrap(), "ex-Tension.");
        assert_err!(Method::try_from("BAD^EXT"));

        let long_method = "This_is_a_very_long_method.It_is_valid_but_unlikely.";
        assert_eq!(Method::try_from(long_method).unwrap(), long_method);
    }
}
