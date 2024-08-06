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
//! assert_eq!(Method::INVITE, Method::try_from("INVITE").unwrap());
//! assert_eq!(Method::BYE.as_str(), "BYE");
//! let method = Method::try_from("CANCEL");
//! assert!(method.is_ok_and(|method| method == Method::CANCEL));
//! ```

use crate::common::header_value_collection::HeaderValueCollection;
use std::{borrow::Cow, hash::Hash, str};

use crate::Error;

/// Representation of the list of methods from an `AllowHeader`.
///
/// This is usable as an iterator.
pub type Methods = HeaderValueCollection<Method>;

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
/// assert_eq!(Method::INVITE, Method::try_from("INVITE").unwrap());
/// assert_eq!(Method::REGISTER.as_str(), "REGISTER");
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Method(Cow<'static, str>);

impl Method {
    /// Return a &str representation of the SIP method.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<&str> for Method {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse(value)
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

fn parse(input: &str) -> Result<Method, Error> {
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
    use nom::{branch::alt, bytes::complete::tag, combinator::map, error::context};
    use std::borrow::Cow;

    #[inline]
    fn ack_method(input: &str) -> ParserResult<&str, Method> {
        tag("ACK")(input).map(|(rest, _)| (rest, Method::ACK))
    }

    #[inline]
    fn bye_method(input: &str) -> ParserResult<&str, Method> {
        tag("BYE")(input).map(|(rest, _)| (rest, Method::BYE))
    }

    #[inline]
    fn cancel_method(input: &str) -> ParserResult<&str, Method> {
        tag("CANCEL")(input).map(|(rest, _)| (rest, Method::CANCEL))
    }

    #[inline]
    fn extension_method(input: &str) -> ParserResult<&str, Method> {
        map(token, |result| Method(Cow::from(result.to_string())))(input)
    }

    #[inline]
    fn invite_method(input: &str) -> ParserResult<&str, Method> {
        tag("INVITE")(input).map(|(rest, _)| (rest, Method::INVITE))
    }

    #[inline]
    fn options_method(input: &str) -> ParserResult<&str, Method> {
        tag("OPTIONS")(input).map(|(rest, _)| (rest, Method::OPTIONS))
    }

    #[inline]
    fn register_method(input: &str) -> ParserResult<&str, Method> {
        tag("REGISTER")(input).map(|(rest, _)| (rest, Method::REGISTER))
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
        assert_err!(Method::try_from(""));
        assert_err!(Method::try_from("\n")); // Invalid method characters
    }

    #[test]
    fn test_valid_method() {
        assert!(Method::try_from("INVITE").is_ok_and(|method| method == Method::INVITE));
        assert!(Method::try_from("CANCEL").is_ok_and(|method| method == Method::CANCEL));
        assert_eq!(Method::INVITE.as_str(), "INVITE");
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
