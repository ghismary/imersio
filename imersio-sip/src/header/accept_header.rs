use std::{collections::HashSet, hash::Hash};

use crate::{
    common::{AcceptParameter, HeaderValueCollection},
    utils::partial_eq_refs,
    HeaderAccessor,
};

use super::generic_header::GenericHeader;

/// Representation of an Accept header.
///
/// The Accept header field follows the same syntax as for HTTP. The semantics
/// are also identical, with the exception that if no Accept header field is
/// present, the server SHOULD assume a default value of `application/sdp`.
///
/// [[RFC3261, Section 20.1](https://datatracker.ietf.org/doc/html/rfc3261#section-20.1)]
#[derive(Clone, Debug, Eq)]
pub struct AcceptHeader {
    header: GenericHeader,
    ranges: AcceptRanges,
}

impl AcceptHeader {
    pub(crate) fn new(header: GenericHeader, ranges: Vec<AcceptRange>) -> Self {
        AcceptHeader {
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
    crate::header::generic_header_accessors!(header);

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

impl std::fmt::Display for AcceptHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for AcceptHeader {
    fn eq(&self, other: &Self) -> bool {
        self.ranges == other.ranges
    }
}

partial_eq_refs!(AcceptHeader);

/// Representation of the list of range from an `AcceptHeader`.
///
/// This is usable as an iterator.
pub type AcceptRanges = HeaderValueCollection<AcceptRange>;

impl AcceptRanges {
    /// Tell whether the ranges contain the given `MediaRange`.
    pub fn contains(&self, media_range: &MediaRange) -> bool {
        self.iter().any(|ar| ar.media_range == media_range)
    }

    /// Get the `Accept-Range` corresponding to the given `MediaRange`.
    pub fn get(&self, media_range: &MediaRange) -> Option<&AcceptRange> {
        self.iter().find(|ar| ar.media_range == media_range)
    }
}

/// Represenation of a range contained in an `AcceptHeader`.
#[derive(Clone, Debug, Eq)]
pub struct AcceptRange {
    media_range: MediaRange,
    parameters: Vec<AcceptParameter>,
}

impl AcceptRange {
    pub(crate) fn new(media_range: MediaRange, parameters: Vec<AcceptParameter>) -> Self {
        AcceptRange {
            media_range,
            parameters,
        }
    }

    /// Get a reference to the `MediaRange` of the `AcceptRange`.
    pub fn media_range(&self) -> &MediaRange {
        &self.media_range
    }

    /// Get a reference to the vector of `AcceptParameter` of the `AcceptRange`.
    pub fn parameters(&self) -> &Vec<AcceptParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for AcceptRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.media_range,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl PartialEq for AcceptRange {
    fn eq(&self, other: &Self) -> bool {
        if self.media_range != other.media_range {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

partial_eq_refs!(AcceptRange);

impl Hash for AcceptRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.media_range.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

/// Representation of a media range contained in an `AcceptRange`.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct MediaRange {
    r#type: String,
    subtype: String,
}

impl MediaRange {
    pub(crate) fn new<S: Into<String>>(r#type: S, subtype: S) -> Self {
        MediaRange {
            r#type: r#type.into(),
            subtype: subtype.into(),
        }
    }
}

impl std::fmt::Display for MediaRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.r#type, self.subtype,)
    }
}

partial_eq_refs!(MediaRange);

#[cfg(test)]
mod tests {
    use super::AcceptHeader;
    use crate::{
        header::accept_header::{AcceptParameter, MediaRange},
        Header, HeaderAccessor,
    };
    use claim::{assert_err, assert_ok};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(AcceptHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert_ok!(&header);
        if let Header::Accept(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not an Accept header");
        }
    }

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

    fn invalid_header(header: &str) {
        assert_err!(Header::from_str(header));
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

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::Accept(first_header), Header::Accept(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Accept header");
        }
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

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::Accept(first_header), Header::Accept(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept header");
        }
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
        let header = Header::from_str(
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
