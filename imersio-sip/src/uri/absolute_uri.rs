use partial_eq_refs::PartialEqRefs;

use crate::{UriHeaders, UriParameters, UriScheme};

/// Representation of an absolute URI.
///
/// As of now, only the scheme is distinguished for the rest of the URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialEqRefs)]
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

    /// Get a reference to the `UriHeaders` of the abosulet uri.
    pub fn headers(&self) -> &UriHeaders {
        &self.headers
    }
}

impl std::fmt::Display for AbsoluteUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.scheme, self.opaque_part)
    }
}
