//! Parsing and generation of an absolute URI.

use crate::parser::{ESCAPED_CHARS, is_reserved, is_unreserved};
use crate::uris::absolute_uri::parser::is_uric_special_char;
use crate::utils::escape;
use crate::{IntoSpecificString, IntoUriScheme, SipError, UriHeaders, UriParameters, UriScheme};

/// Representation of a URI user value accepting only the valid characters.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, derive_more::Deref, derive_more::Display)]
pub struct OpaquePartString(String);

impl OpaquePartString {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl TryFrom<&str> for OpaquePartString {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Do not use the parser because of the escaped characters, instead check that each
        // character of the given value can be escaped.
        if !value.is_empty()
            && value.chars().all(|c| {
                let idx: Result<u8, _> = c.try_into();
                match idx {
                    Ok(idx) => ESCAPED_CHARS[idx as usize] != '\0',
                    Err(_) => false,
                }
            })
        {
            Ok(Self::new(value))
        } else {
            Err(SipError::InvalidUriOpaquePart(value.to_string()))
        }
    }
}

/// Representation of an absolute URI.
///
/// As of now, only the scheme is distinguished for the rest of the URI.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct AbsoluteUri {
    scheme: UriScheme,
    opaque_part: OpaquePartString,
    parameters: UriParameters,
    headers: UriHeaders,
}

impl AbsoluteUri {
    pub(crate) fn new(
        scheme: UriScheme,
        opaque_part: OpaquePartString,
        parameters: UriParameters,
        headers: UriHeaders,
    ) -> Self {
        Self {
            scheme,
            opaque_part,
            parameters,
            headers,
        }
    }

    /// Get a reference to the `UriScheme` of the absolute uri.
    pub fn scheme(&self) -> &UriScheme {
        &self.scheme
    }

    /// Get the opaque part of the absolute uri.
    pub fn opaque_part(&self) -> &str {
        &self.opaque_part
    }

    /// Get a `AbsoluteUri` builder.
    pub fn builder() -> AbsoluteUriBuilder {
        AbsoluteUriBuilder::default()
    }

    /// Get a reference to the `UriParameters` of the absolute uri.
    pub(crate) fn parameters(&self) -> &UriParameters {
        &self.parameters
    }

    /// Get a reference to the `UriHeaders` of the absolute uri.
    pub(crate) fn headers(&self) -> &UriHeaders {
        &self.headers
    }
}

impl std::fmt::Display for AbsoluteUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}",
            self.scheme,
            escape(self.opaque_part(), |c| {
                is_reserved(c) || is_unreserved(c) || is_uric_special_char(c)
            })
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct AbsoluteUriBuilder {
    scheme: Option<UriScheme>,
    opaque_part: Option<OpaquePartString>,
}

impl AbsoluteUriBuilder {
    pub fn try_scheme<S: Into<IntoUriScheme>>(&mut self, scheme: S) -> Result<&mut Self, SipError> {
        let scheme = scheme.into();
        let scheme = scheme.try_into()?;
        match scheme {
            UriScheme::Sip | UriScheme::Sips => Err(SipError::InvalidUriScheme(scheme.to_string())),
            UriScheme::Other(_) => {
                self.scheme = Some(scheme);
                Ok(self)
            }
        }
    }

    pub fn try_opaque_part<O: Into<IntoSpecificString<OpaquePartString>>>(
        &mut self,
        opaque_part: O,
    ) -> Result<&mut Self, SipError> {
        let opaque_part = opaque_part.into();
        let opaque_part = opaque_part.try_into()?;
        self.opaque_part = Some(opaque_part);
        Ok(self)
    }

    pub fn try_build(&self) -> Result<AbsoluteUri, SipError> {
        let mut uri = AbsoluteUri::default();
        match &self.scheme {
            Some(scheme) => uri.scheme = Clone::clone(scheme),
            None => {
                return Err(SipError::InvalidUriScheme(
                    "No scheme given to the builder".to_string(),
                ));
            }
        }
        match &self.opaque_part {
            Some(opaque_part) => uri.opaque_part = Clone::clone(opaque_part),
            None => {
                return Err(SipError::InvalidUriOpaquePart(
                    "No opaque part given to the builder".to_string(),
                ));
            }
        }
        Ok(uri)
    }
}

pub(crate) mod parser {
    use nom::{
        Parser,
        branch::alt,
        bytes::complete::tag,
        combinator::{map, recognize, verify},
        error::context,
        multi::many0,
        sequence::{pair, separated_pair},
    };

    use crate::{
        AbsoluteUri, OpaquePartString, UriHeaders, UriParameters, UriScheme,
        parser::{ParserResult, escaped, is_reserved, is_unreserved, reserved, take1, unreserved},
        uris::uri_scheme::parser::scheme,
    };

    #[inline]
    pub(super) fn is_uric_special_char(c: char) -> bool {
        ";?:@&=+$,".contains(c)
    }

    #[inline]
    fn uric(input: &str) -> ParserResult<&str, char> {
        alt((reserved, unreserved, escaped)).parse(input)
    }

    #[inline]
    fn uric_no_slash(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| {
            is_reserved(*c) || is_unreserved(*c) || is_uric_special_char(*c)
        })
        .parse(input)
    }

    #[inline]
    fn opaque_part(input: &str) -> ParserResult<&str, OpaquePartString> {
        map(
            recognize(pair(uric_no_slash, many0(uric))),
            OpaquePartString::new,
        )
        .parse(input)
    }

    pub(crate) fn absolute_uri(input: &str) -> ParserResult<&str, AbsoluteUri> {
        context(
            "absolute_uri",
            map(
                separated_pair(scheme, tag(":"), opaque_part),
                |(scheme, opaque_part)| {
                    AbsoluteUri::new(
                        UriScheme::Other(scheme),
                        opaque_part,
                        UriParameters::default(),
                        UriHeaders::default(),
                    )
                },
            ),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AbsoluteUri, UriScheme, UriSchemeString};
    use claims::assert_err;

    #[test]
    fn test_valid_absolute_uri_builder() {
        let uri = AbsoluteUri::builder()
            .try_scheme("http")
            .unwrap()
            .try_opaque_part("//localhost")
            .unwrap()
            .try_build()
            .unwrap();
        assert_eq!(
            uri.scheme(),
            &UriScheme::Other(UriSchemeString::new("http"))
        );
        assert_eq!(uri.to_string(), "http://localhost");
    }

    #[test]
    fn test_invalid_absolute_uri_with_sip_scheme_builder() {
        assert_err!(AbsoluteUri::builder().try_scheme(UriScheme::Sip));
    }

    #[test]
    fn test_invalid_absolute_uri_with_invalid_character_in_opaque_part_builder() {
        assert_err!(AbsoluteUri::builder().try_opaque_part("😁"));
    }

    #[test]
    fn test_invalid_absolute_uri_missing_scheme_builder() {
        let mut builder = AbsoluteUri::builder();
        builder.try_opaque_part("//example.com").unwrap();
        assert_err!(builder.try_build());
    }

    #[test]
    fn test_invalid_absolute_uri_missing_opaque_part_builder() {
        let mut builder = AbsoluteUri::builder();
        builder.try_scheme("file").unwrap();
        assert_err!(builder.try_build());
    }
}
