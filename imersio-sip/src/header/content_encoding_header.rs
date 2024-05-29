use partial_eq_refs::PartialEqRefs;

use crate::{
    common::{content_encoding::ContentEncoding, header_value_collection::HeaderValueCollection},
    HeaderAccessor,
};

use super::generic_header::GenericHeader;

/// Representation of a Content-Encoding header.
///
/// The Content-Encoding header field is used as a modifier to the
/// "media-type". When present, its value indicates what additional content
/// codings have been applied to the entity-body, and thus what decoding
/// mechanisms MUST be applied in order to obtain the media-type referenced
/// by the Content-Type header field. Content-Encoding is primarily used to
/// allow a body to be compressed without losing the identity of its
/// underlying media type.
///
/// If multiple encodings have been applied to an entity-body, the content
/// codings MUST be listed in the order in which they were applied.
///
/// [[RFC3261, Section 20.12](https://datatracker.ietf.org/doc/html/rfc3261#section-20.12)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ContentEncodingHeader {
    header: GenericHeader,
    encodings: ContentEncodings,
}

impl ContentEncodingHeader {
    pub(crate) fn new(header: GenericHeader, encodings: Vec<ContentEncoding>) -> Self {
        ContentEncodingHeader {
            header,
            encodings: encodings.into(),
        }
    }

    /// Get a reference to the encodings from the Content-Encoding header.
    pub fn encodings(&self) -> &ContentEncodings {
        &self.encodings
    }
}

impl HeaderAccessor for ContentEncodingHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("e")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Content-Encoding")
    }
    fn normalized_value(&self) -> String {
        self.encodings.to_string()
    }
}

impl std::fmt::Display for ContentEncodingHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq<ContentEncodingHeader> for ContentEncodingHeader {
    fn eq(&self, other: &ContentEncodingHeader) -> bool {
        self.encodings == other.encodings
    }
}

/// Representation of the list of encodings in a `Content-Encoding` header.
///
/// This is usable as an iterator.
pub type ContentEncodings = HeaderValueCollection<ContentEncoding>;

#[cfg(test)]
mod tests {
    use super::ContentEncodingHeader;
    use crate::{Header, HeaderAccessor};
    use claim::{assert_err, assert_ok};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(ContentEncodingHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert_ok!(&header);
        if let Header::ContentEncoding(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not a Content-Encoding header");
        }
    }

    #[test]
    fn test_valid_content_encoding_header() {
        valid_header("Content-Encoding: gzip", |header| {
            assert_eq!(header.encodings().len(), 1);
            assert_eq!(header.encodings().first().unwrap(), "gzip");
        });
    }

    #[test]
    fn test_valid_content_encoding_header_in_compact_form() {
        valid_header("e: tar", |header| {
            assert_eq!(header.encodings().len(), 1);
            assert_eq!(header.encodings().first().unwrap(), "tar");
        });
    }

    fn invalid_header(header: &str) {
        assert_err!(Header::from_str(header));
    }

    #[test]
    fn test_invalid_content_encoding_header_empty() {
        invalid_header("Content-Encoding:");
    }

    #[test]
    fn test_invalid_content_encoding_header_empty_with_space_characters() {
        invalid_header("Content-Encoding:    ");
    }

    #[test]
    fn test_invalid_content_encoding_header_with_invalid_character() {
        invalid_header("Content-Encoding: 😁");
    }

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::ContentEncoding(first_header), Header::ContentEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not a Content-Encoding header");
        }
    }

    #[test]
    fn test_content_encoding_header_equality_same_header_with_space_characters_differences() {
        header_equality("Content-Encoding: gzip", "Content-Encoding:  gzip");
    }

    #[test]
    fn test_content_encoding_header_equality_same_encodings_in_a_different_order() {
        header_equality("Content-Encoding: gzip, tar", "Content-Encoding: tar, gzip");
    }

    #[test]
    fn test_content_encoding_header_equality_same_encodings_with_different_cases() {
        header_equality("Content-Encoding: gzip", "content-encoding: GZIP");
    }

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::ContentEncoding(first_header), Header::ContentEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not a Content-Encoding header");
        }
    }

    #[test]
    fn test_content_encoding_header_inequality_with_different_encodings() {
        header_inequality("Content-Encoding: gzip", "Content-Encoding: tar");
    }

    #[test]
    fn test_content_encoding_header_inequality_with_first_having_more_encodings_than_the_second() {
        header_inequality("Content-Encoding: gzip, tar", "Content-Encoding: tar");
    }

    #[test]
    fn test_content_encoding_header_inequality_with_first_having_less_encodings_than_the_second() {
        header_inequality("Content-Encoding: gzip", "Content-Encoding: tar, gzip");
    }

    #[test]
    fn test_content_encoding_header_to_string() {
        let header = Header::from_str("content-enCoding:  tar , GZIP");
        if let Header::ContentEncoding(header) = header.unwrap() {
            assert_eq!(header.to_string(), "content-enCoding:  tar , GZIP");
            assert_eq!(header.to_normalized_string(), "Content-Encoding: tar, gzip");
            assert_eq!(header.to_compact_string(), "e: tar, gzip");
        }
    }
}
