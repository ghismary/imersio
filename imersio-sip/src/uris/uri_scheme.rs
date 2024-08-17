//! Parsing and generation of the scheme of a SIP URI.

use std::hash::Hash;

use derive_more::{Deref, Display};
use nom::error::convert_error;

use crate::uris::uri_scheme::parser::scheme;
use crate::SipError;

/// Representation of a URI scheme value accepting only the valid characters.
#[derive(Clone, Debug, Deref, Display, Eq, Hash, PartialEq)]
pub struct UriSchemeToken(String);

impl UriSchemeToken {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl TryFrom<&str> for UriSchemeToken {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match scheme(value) {
            Ok((rest, scheme_token)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(scheme_token)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidUriScheme(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidUriScheme(format!(
                "Incomplete uri scheme `{}`",
                value
            ))),
        }
    }
}

/// Representation of the scheme of a URI.
#[derive(Clone, Debug, Eq)]
pub enum UriScheme {
    /// SIP protocol scheme.
    Sip,
    /// SIPS protocol scheme.
    Sips,
    /// Any other protocol scheme.
    Other(UriSchemeToken),
}

impl UriScheme {
    /// SIP protocol scheme.
    pub const SIP: UriScheme = UriScheme::Sip;

    /// SIPS protocol scheme.
    pub const SIPS: UriScheme = UriScheme::Sips;

    /// Get a str representation of the scheme.
    pub fn as_str(&self) -> &str {
        match self {
            UriScheme::Sip => "sip",
            UriScheme::Sips => "sips",
            UriScheme::Other(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for UriScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for UriScheme {
    fn default() -> Self {
        UriScheme::SIP
    }
}

impl AsRef<str> for UriScheme {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for UriScheme {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (&UriScheme::Sip, &UriScheme::Sip) => true,
            (&UriScheme::Sips, &UriScheme::Sips) => true,
            (UriScheme::Other(a), UriScheme::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialEq<str> for UriScheme {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<UriScheme> for str {
    fn eq(&self, other: &UriScheme) -> bool {
        other == self
    }
}

impl Hash for UriScheme {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            UriScheme::Sip => {
                state.write_u8(1);
            }
            UriScheme::Sips => {
                state.write_u8(2);
            }
            UriScheme::Other(value) => {
                state.write_u8(3);
                value.to_ascii_lowercase().hash(state);
            }
        }
    }
}

impl TryFrom<&str> for UriScheme {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lowercase_value = value.to_lowercase();
        match lowercase_value.as_str() {
            "sip" => Ok(UriScheme::Sip),
            "sips" => Ok(UriScheme::Sips),
            _ => UriSchemeToken::try_from(value).map(UriScheme::Other),
        }
    }
}

pub(crate) mod parser {
    use crate::parser::{alpha, digit, take1, ParserResult};
    use crate::UriSchemeToken;
    use nom::{
        branch::alt,
        combinator::{map, recognize, verify},
        error::context,
        multi::many0,
        sequence::pair,
    };

    #[inline]
    fn scheme_special_char(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| "+-.".contains(*c))(input)
    }

    pub(crate) fn scheme(input: &str) -> ParserResult<&str, UriSchemeToken> {
        context(
            "scheme",
            map(
                recognize(pair(alpha, many0(alt((alpha, digit, scheme_special_char))))),
                UriSchemeToken::new,
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{UriScheme, UriSchemeToken};
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_valid_uri_scheme_token() {
        let scheme_token = UriSchemeToken::try_from("http");
        assert_ok!(&scheme_token);
        if let Ok(scheme_token) = scheme_token {
            assert_eq!(scheme_token.as_str(), "http");
            assert_eq!(format!("{}", scheme_token), "http");
        }
    }

    #[test]
    fn test_invalid_uri_scheme_token() {
        assert_err!(UriSchemeToken::try_from("my_scheme"));
    }

    #[test]
    fn test_valid_uri_scheme_sip() {
        let scheme = UriScheme::try_from("sip");
        assert_ok!(&scheme);
        if let Ok(scheme) = scheme {
            assert_eq!(scheme, UriScheme::Sip);
            assert_eq!(format!("{}", scheme), "sip");
        }
    }

    #[test]
    fn test_valid_uri_scheme_sips() {
        let scheme = UriScheme::try_from("SIPS");
        assert_ok!(&scheme);
        if let Ok(scheme) = scheme {
            assert_eq!(scheme, UriScheme::Sips);
            assert_eq!(format!("{}", scheme), "sips");
        }
    }

    #[test]
    fn test_valid_uri_scheme_http() {
        let scheme = UriScheme::try_from("http");
        assert_ok!(&scheme);
        if let Ok(scheme) = scheme {
            assert_eq!(
                scheme,
                UriScheme::Other(UriSchemeToken::try_from("http").unwrap())
            );
            assert_eq!(format!("{}", scheme), "http");
        }
    }

    #[test]
    fn test_invalid_uri_scheme() {
        assert_err!(UriScheme::try_from("@sch&me"));
    }
}
