//! SIP Error-Info header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{ErrorUri, ErrorUris};

/// Representation of an Error-Info header.
///
/// The Error-Info header field provides a pointer to additional information about the error status
/// response.
/// [[RFC3261, Section 20.18](https://datatracker.ietf.org/doc/html/rfc3261#section-20.18)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct ErrorInfoHeader {
    #[partial_eq_ignore]
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
    crate::headers::generic_header_accessors!(header);

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

pub(crate) mod parser {
    use crate::common::error_uri::parser::error_uri;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{ErrorInfoHeader, Header};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn error_info(input: &str) -> ParserResult<&str, Header> {
        context(
            "Error-Info header",
            map(
                tuple((
                    tag_no_case("Error-Info"),
                    hcolon,
                    cut(consumed(separated_list1(comma, error_uri))),
                )),
                |(name, separator, (value, uris))| {
                    Header::ErrorInfo(ErrorInfoHeader::new(
                        GenericHeader::new(name, separator, value),
                        uris,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        ErrorInfoHeader, Header, Uri,
    };
    use claims::assert_ok;

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
                    .contains(&Uri::try_from("sip:not-in-service-recording@atlanta.com").unwrap()));
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
        let header = Header::try_from(
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