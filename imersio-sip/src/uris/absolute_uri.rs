//! Parsing and generation of an absolute URI.

use crate::{UriHeaders, UriParameters, UriScheme};

/// Representation of an absolute URI.
///
/// As of now, only the scheme is distinguished for the rest of the URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AbsoluteUri {
    scheme: UriScheme,
    opaque_part: String,
    parameters: UriParameters,
    headers: UriHeaders,
}

impl AbsoluteUri {
    pub(crate) fn new<S: Into<String>>(
        scheme: UriScheme,
        opaque_part: S,
        parameters: UriParameters,
        headers: UriHeaders,
    ) -> Self {
        Self {
            scheme,
            opaque_part: opaque_part.into(),
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

    /// Get a reference to the `UriParameters` of the absolute uri.
    pub fn parameters(&self) -> &UriParameters {
        &self.parameters
    }

    /// Get a reference to the `UriHeaders` of the absolute uri.
    pub fn headers(&self) -> &UriHeaders {
        &self.headers
    }
}

impl std::fmt::Display for AbsoluteUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.scheme, self.opaque_part)
    }
}

pub(crate) mod parser {
    use crate::parser::{
        escaped, is_reserved, is_unreserved, reserved, take1, unreserved, ParserResult,
    };
    use crate::uris::uri_scheme::parser::scheme;
    use crate::{AbsoluteUri, UriHeaders, UriParameters, UriScheme};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, recognize, verify},
        error::context,
        multi::many0,
        sequence::{pair, separated_pair},
    };

    fn uric(input: &str) -> ParserResult<&str, char> {
        alt((reserved, unreserved, escaped))(input)
    }

    fn uric_no_slash(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| {
            is_reserved(*c) || is_unreserved(*c) || ";?:@&=+$,".contains(*c)
        })(input)
    }

    fn opaque_part(input: &str) -> ParserResult<&str, &str> {
        recognize(pair(uric_no_slash, many0(uric)))(input)
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
        )(input)
    }
}
