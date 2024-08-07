//! SIP Accept header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{AcceptRange, AcceptRanges};

/// Representation of an Accept header.
///
/// The Accept header field follows the same syntax as for HTTP. The semantics
/// are also identical, with the exception that if no Accept header field is
/// present, the server SHOULD assume a default value of `application/sdp`.
///
/// [[RFC3261, Section 20.1](https://datatracker.ietf.org/doc/html/rfc3261#section-20.1)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct AcceptHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    ranges: AcceptRanges,
}

impl AcceptHeader {
    pub(crate) fn new(header: GenericHeader, ranges: Vec<AcceptRange>) -> Self {
        Self {
            header,
            ranges: ranges.into(),
        }
    }

    /// Get a reference to the ranges from the `Accept` header.
    pub fn ranges(&self) -> &AcceptRanges {
        &self.ranges
    }
}

impl HeaderAccessor for AcceptHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Accept")
    }
    fn normalized_value(&self) -> String {
        self.ranges.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::media_range::MediaRange,
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        AcceptHeader, AcceptParameter, Header,
    };
    use claims::assert_ok;

    valid_header!(Accept, AcceptHeader, "Accept");
    header_equality!(Accept, "Accept");
    header_inequality!(Accept, "Accept");

    #[test]
    fn test_valid_accept_header_with_single_range() {
        valid_header("Accept: application/sdp", |header| {
            assert!(!header.ranges().is_empty());
            assert_eq!(header.ranges().len(), 1);
            assert!(header
                .ranges()
                .contains(&MediaRange::new("application", "sdp")));
            assert!(!header
                .ranges()
                .contains(&MediaRange::new("application", "x-private")));
            assert!(!header.ranges().contains(&MediaRange::new("text", "html")));
        });
    }

    #[test]
    fn test_valid_accept_header_with_several_ranges() {
        valid_header(
            "Accept: application/sdp;level=1, application/x-private, text/html",
            |header| {
                assert!(!header.ranges().is_empty());
                assert_eq!(header.ranges().len(), 3);
                assert!(header
                    .ranges()
                    .contains(&MediaRange::new("application", "sdp")));
                assert!(header
                    .ranges()
                    .contains(&MediaRange::new("application", "x-private")));
                assert!(header.ranges().contains(&MediaRange::new("text", "html")));
                let accept_range = header.ranges().get(&MediaRange::new("application", "sdp"));
                assert!(accept_range.is_some());
                let accept_range = accept_range.unwrap();
                assert_eq!(accept_range.parameters().len(), 1);
                assert_eq!(
                    accept_range.parameters().first().unwrap(),
                    AcceptParameter::new("level", Some("1"))
                );
                let accept_range = header.ranges().get(&MediaRange::new("text", "html"));
                assert!(accept_range.is_some());
                let accept_range = accept_range.unwrap();
                assert!(accept_range.parameters().is_empty());
            },
        );
    }

    #[test]
    fn test_valid_accept_header_with_wildcard_range() {
        valid_header("Accept: */*", |header| {
            assert!(!header.ranges().is_empty());
            assert_eq!(header.ranges().len(), 1);
            assert!(header.ranges().contains(&MediaRange::new("*", "*")));
        });
    }

    #[test]
    fn test_valid_accept_header_with_wildcard_subtype_range() {
        valid_header("Accept: text/*", |header| {
            assert!(!header.ranges().is_empty());
            assert_eq!(header.ranges().len(), 1);
            assert!(header.ranges().contains(&MediaRange::new("text", "*")));
        });
    }

    #[test]
    fn test_valid_accept_header_empty() {
        valid_header("Accept:", |header| {
            assert!(header.ranges().is_empty());
            assert_eq!(header.ranges().len(), 0);
            assert!(!header
                .ranges()
                .contains(&MediaRange::new("application", "sdp")));
            assert!(!header.ranges().contains(&MediaRange::new("text", "html")));
        });
    }

    #[test]
    fn test_valid_accept_header_empty_with_space_characters() {
        valid_header("Accept:  ", |header| {
            assert!(header.ranges().is_empty());
            assert_eq!(header.ranges().len(), 0);
            assert!(!header
                .ranges()
                .contains(&MediaRange::new("application", "sdp")));
            assert!(!header.ranges().contains(&MediaRange::new("text", "html")));
        });
    }

    #[test]
    fn test_invalid_accept_header_only_range_type() {
        invalid_header("Accept: application");
    }

    #[test]
    fn test_invalid_accept_header_only_range_type_and_slash() {
        invalid_header("Accept: application/");
    }

    #[test]
    fn test_invalid_accept_header_invalid_characters() {
        invalid_header("Accept: 😁/😁");
    }

    #[test]
    fn test_accept_header_equality_same_headers_with_just_space_characters_differences() {
        header_equality("Accept: text/html", "Accept:  text/html");
    }

    #[test]
    fn test_accept_header_equality_same_headers_with_different_ranges_order() {
        header_equality(
            "Accept: text/html, application/sdp",
            "Accept: application/sdp, text/html",
        );
    }

    #[test]
    fn test_accept_header_inequality_with_different_ranges() {
        header_inequality("Accept: application/sdp", "Accept: text/html");
    }

    #[test]
    fn test_accept_header_inequality_with_first_header_having_more_ranges_than_the_second() {
        header_inequality("Accept: application/sdp, text/html", "Accept: text/html");
    }

    #[test]
    fn test_accept_header_inequality_with_first_header_having_less_ranges_than_the_second() {
        header_inequality("Accept: text/html", "Accept: application/sdp, text/html");
    }

    #[test]
    fn test_accept_header_to_string() {
        let header = Header::try_from(
            "accept:   application/sdp ; level =1 , application/x-private   ,  text/html",
        );
        if let Header::Accept(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "accept:   application/sdp ; level =1 , application/x-private   ,  text/html"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Accept: application/sdp;level=1, application/x-private, text/html"
            );
            assert_eq!(
                header.to_compact_string(),
                "Accept: application/sdp;level=1, application/x-private, text/html"
            );
        }
    }
}
