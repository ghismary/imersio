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
//! use std::str::FromStr;
//!
//! assert_eq!(Method::INVITE, Method::from_bytes(b"INVITE").unwrap());
//! assert_eq!(Method::BYE.as_str(), "BYE");
//! let method = Method::from_str("CANCEL");
//! assert!(method.is_ok_and(|method| method == Method::CANCEL));
//! ```

use std::{
    borrow::Cow,
    hash::Hash,
    str::{self, FromStr},
};

use crate::Error;

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
/// assert_eq!(Method::INVITE, Method::from_bytes(b"INVITE").unwrap());
/// assert_eq!(Method::REGISTER.as_str(), "REGISTER");
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Method(Cow<'static, str>);

impl Method {
    /// Return a &str representation of the SIP method.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Try to create a `Method` from a slice of bytes.
    #[inline]
    pub fn from_bytes(input: &[u8]) -> Result<Method, Error> {
        parse(input)
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Method::from_bytes(s.as_bytes())
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl Default for Method {
    #[inline]
    fn default() -> Self {
        Method::INVITE
    }
}

impl AsRef<str> for Method {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&Method> for Method {
    #[inline]
    fn eq(&self, other: &&Method) -> bool {
        self == *other
    }
}

impl PartialEq<Method> for &Method {
    #[inline]
    fn eq(&self, other: &Method) -> bool {
        *self == other
    }
}

impl PartialEq<str> for Method {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Method> for str {
    #[inline]
    fn eq(&self, other: &Method) -> bool {
        self == other.as_ref()
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

macro_rules! methods {
    (
        $(
            $(#[$docs:meta])*
            ($konst:ident, $word:expr),
        )+
    ) => {
        impl Method {
            $(
                $(#[$docs])*
                pub const $konst: Method = Method(Cow::Borrowed($word));
            )+
        }
    }
}

methods! {
    /// ACK method
    (ACK, "ACK"),
    /// BYE method
    (BYE, "BYE"),
    /// CANCEL method
    (CANCEL, "CANCEL"),
    /// INVITE method
    (INVITE, "INVITE"),
    /// OPTIONS method
    (OPTIONS, "OPTIONS"),
    /// REGISTER method
    (REGISTER, "REGISTER"),
}

fn parse(input: &[u8]) -> Result<Method, Error> {
    match parser::method(input) {
        Ok((rest, method)) => {
            if !rest.is_empty() {
                Err(Error::RemainingUnparsedData)
            } else {
                Ok(method)
            }
        }
        Err(e) => Err(Error::InvalidMethod(e.to_string())),
    }
}

pub(crate) mod parser {
    use super::Method;
    use crate::parser::*;
    use nom::{branch::alt, bytes::complete::tag, error::context};
    use std::borrow::Cow;

    #[inline]
    fn ack_method(input: &[u8]) -> ParserResult<&[u8], Method> {
        tag("ACK")(input).map(|(rest, _)| (rest, Method::ACK))
    }

    #[inline]
    fn bye_method(input: &[u8]) -> ParserResult<&[u8], Method> {
        tag("BYE")(input).map(|(rest, _)| (rest, Method::BYE))
    }

    #[inline]
    fn cancel_method(input: &[u8]) -> ParserResult<&[u8], Method> {
        tag("CANCEL")(input).map(|(rest, _)| (rest, Method::CANCEL))
    }

    #[inline]
    fn extension_method(input: &[u8]) -> ParserResult<&[u8], Method> {
        token(input).map(|(rest, result)| (rest, Method(Cow::Owned(result.into_owned()))))
    }

    #[inline]
    fn invite_method(input: &[u8]) -> ParserResult<&[u8], Method> {
        tag("INVITE")(input).map(|(rest, _)| (rest, Method::INVITE))
    }

    #[inline]
    fn options_method(input: &[u8]) -> ParserResult<&[u8], Method> {
        tag("OPTIONS")(input).map(|(rest, _)| (rest, Method::OPTIONS))
    }

    #[inline]
    fn register_method(input: &[u8]) -> ParserResult<&[u8], Method> {
        tag("REGISTER")(input).map(|(rest, _)| (rest, Method::REGISTER))
    }

    pub(crate) fn method(input: &[u8]) -> ParserResult<&[u8], Method> {
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
        )(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::assert_err;

    #[test]
    fn test_method_eq() {
        assert_eq!(Method::INVITE, "INVITE");
        assert_eq!(&Method::INVITE, "INVITE");

        assert_eq!("INVITE", Method::INVITE);
        assert_eq!("INVITE", &Method::INVITE);

        assert_eq!(&Method::INVITE, Method::INVITE);
        assert_eq!(Method::INVITE, &Method::INVITE);
    }

    #[test]
    fn test_invalid_method() {
        assert_err!(Method::from_str(""));
        assert_err!(Method::from_bytes(b""));
        assert_err!(Method::from_bytes(&[0xC0])); // Invalid UTF-8
        assert_err!(Method::from_bytes(&[0x10])); // Invalid method characters
    }

    #[test]
    fn test_valid_method() {
        assert!(Method::from_str("INVITE").is_ok_and(|method| method == Method::INVITE));
        assert!(Method::from_bytes(b"CANCEL").is_ok_and(|method| method == Method::CANCEL));
        assert_eq!(Method::INVITE.as_str(), "INVITE");
    }

    #[test]
    fn test_extension_method() {
        assert_eq!(Method::from_bytes(b"EXTENSION").unwrap(), "EXTENSION");
        assert_eq!(Method::from_bytes(b"ex-Tension.").unwrap(), "ex-Tension.");
        assert_err!(Method::from_bytes(b"BAD^EXT"));

        let long_method = "This_is_a_very_long_method.It_is_valid_but_unlikely.";
        assert_eq!(Method::from_str(long_method).unwrap(), long_method);
    }
}
