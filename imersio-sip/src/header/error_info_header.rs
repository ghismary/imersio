use std::{collections::HashSet, hash::Hash};

use partial_eq_refs::PartialEqRefs;

use crate::{
    common::header_value_collection::HeaderValueCollection, GenericParameter, HeaderAccessor, Uri,
};

use super::generic_header::GenericHeader;

/// Representation of an Error-Info header.
///
/// The Error-Info header field provides a pointer to additional information about the error status
/// response.
/// [[RFC3261, Section 20.18](https://datatracker.ietf.org/doc/html/rfc3261#section-20.18)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ErrorInfoHeader {
    header: GenericHeader,
    error_uris: ErrorUris,
}

impl ErrorInfoHeader {
    pub(crate) fn new(header: GenericHeader, error_uris: Vec<ErrorUri>) -> Self {
        Self {
            header,
            error_uris: error_uris.into(),
        }
    }

    /// Get a reference to the error uris from the `ErrorInfoHeader`.
    pub fn error_uris(&self) -> &ErrorUris {
        &self.error_uris
    }
}

impl HeaderAccessor for ErrorInfoHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Error-Info")
    }
    fn normalized_value(&self) -> String {
        self.error_uris.to_string()
    }
}

impl std::fmt::Display for ErrorInfoHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for ErrorInfoHeader {
    fn eq(&self, other: &Self) -> bool {
        self.error_uris == other.error_uris
    }
}

/// Representation of the list of error uris from an `ErrorInfoHeader`.
///
/// This is usable as an iterator.
pub type ErrorUris = HeaderValueCollection<ErrorUri>;

impl ErrorUris {
    /// Tell whether `ErrorUris` contain the given `Uri`.
    pub fn contains(&self, uri: &Uri) -> bool {
        self.iter().any(|a| a.uri == uri)
    }

    /// Get the `ErrorUri` corresponding to the given `Uri`.
    pub fn get(&self, uri: &Uri) -> Option<&ErrorUri> {
        self.iter().find(|a| a.uri == uri)
    }
}

/// Representation of an error uri contained in an `Error-Info` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ErrorUri {
    uri: Uri,
    parameters: Vec<GenericParameter>,
}

impl ErrorUri {
    pub(crate) fn new(uri: Uri, parameters: Vec<GenericParameter>) -> Self {
        ErrorUri { uri, parameters }
    }

    /// Get a reference to the uri contained in the `ErrorUri`.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Get a reference to the parameters contained in the `ErrorUri`.
    pub fn parameters(&self) -> &Vec<GenericParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for ErrorUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}>{}{}",
            self.uri,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl PartialEq for ErrorUri {
    fn eq(&self, other: &Self) -> bool {
        if self.uri != other.uri {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

impl Hash for ErrorUri {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorInfoHeader;
    use crate::{
        header::tests::{header_equality, header_inequality, invalid_header, valid_header},
        Header, HeaderAccessor, Uri,
    };
    use claims::assert_ok;
    use std::str::FromStr;

    valid_header!(ErrorInfo, ErrorInfoHeader, "Error-Info");
    header_equality!(ErrorInfo, "Error-Info");
    header_inequality!(ErrorInfo, "Error-Info");

    #[test]
    fn test_valid_error_info_header() {
        valid_header(
            "Error-Info: <sip:not-in-service-recording@atlanta.com>",
            |header| {
                assert_eq!(header.error_uris().len(), 1);
                assert!(header
                    .error_uris()
                    .contains(&Uri::from_str("sip:not-in-service-recording@atlanta.com").unwrap()));
            },
        );
    }

    #[test]
    fn test_invalid_error_info_header_empty() {
        invalid_header("Error-Info:");
    }

    #[test]
    fn test_invalid_error_info_header_empty_with_space_characters() {
        invalid_header("Error-Info:       ");
    }

    #[test]
    fn test_invalid_error_info_header_missing_brackets_around_the_uri() {
        invalid_header("Error-Info: sip:not-in-service-recording@atlanta.com");
    }

    #[test]
    fn test_error_info_header_equality_with_space_characters_differences() {
        header_equality(
            "Error-Info: <sip:not-in-service-recording@atlanta.com>",
            "Error-Info :   <sip:not-in-service-recording@atlanta.com>",
        );
    }

    #[test]
    fn test_error_info_header_equality_with_same_uri_and_same_parameters_with_different_cases() {
        header_equality(
            "Error-Info: <sip:not-in-service-recording@atlanta.com>;myparam=test",
            "Error-Info: <sip:not-in-service-recording@atlanta.com> ;MyParam=TEST",
        );
    }

    #[test]
    fn test_error_info_header_inequality_with_different_uris() {
        header_inequality(
            "Error-Info: <sip:not-in-service-recording@atlanta.com>",
            "Error-Info: <sip:not-in-service-recording@vancouver.com>",
        );
    }

    #[test]
    fn test_error_info_header_inequality_with_same_uri_but_different_parameters() {
        header_inequality(
            "Error-Info: <sip:not-in-service-recording@atlanta.com>;foo=bar",
            "Error-Info: <sip:not-in-service-recording@atlanta.com>;foo=baz",
        );
    }

    #[test]
    fn test_error_info_header_to_string() {
        let header = Header::from_str(
            "errOr-infO:    <sip:not-in-service-recording@atlanta.com> ; MyparaM = Test",
        );
        if let Header::ErrorInfo(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "errOr-infO:    <sip:not-in-service-recording@atlanta.com> ; MyparaM = Test"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Error-Info: <sip:not-in-service-recording@atlanta.com>;myparam=test"
            );
            assert_eq!(
                header.to_compact_string(),
                "Error-Info: <sip:not-in-service-recording@atlanta.com>;myparam=test"
            );
        }
    }
}
