use partial_eq_refs::PartialEqRefs;

use super::{generic_header::GenericHeader, HeaderAccessor};

/// Representation of a Content-Length header.
///
/// [[RFC3261, Section 20.14](https://datatracker.ietf.org/doc/html/rfc3261#section-20.14)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ContentLengthHeader {
    header: GenericHeader,
    content_length: u32,
}

impl ContentLengthHeader {
    pub(crate) fn new(header: GenericHeader, content_length: u32) -> Self {
        ContentLengthHeader {
            header,
            content_length,
        }
    }

    /// Get the content length from the Content-Length header.
    pub fn content_length(&self) -> u32 {
        self.content_length
    }
}

impl HeaderAccessor for ContentLengthHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("l")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Content-Length")
    }
    fn normalized_value(&self) -> String {
        format!("{}", self.content_length)
    }
}

impl std::fmt::Display for ContentLengthHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq<ContentLengthHeader> for ContentLengthHeader {
    fn eq(&self, other: &ContentLengthHeader) -> bool {
        self.content_length == other.content_length
    }
}

#[cfg(test)]
mod tests {
    use super::ContentLengthHeader;
    use crate::{header::HeaderAccessor, Header};
    use claim::{assert_err, assert_ok};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(ContentLengthHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert_ok!(&header);
        if let Header::ContentLength(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not a Content-Length header");
        }
    }

    #[test]
    fn test_valid_content_length_header() {
        valid_header("Content-Length: 349", |header| {
            assert_eq!(header.content_length(), 349);
        });
    }

    #[test]
    fn test_valid_content_length_header_in_compact_form() {
        valid_header("l: 173", |header| {
            assert_eq!(header.content_length(), 173);
        });
    }

    fn invalid_header(header: &str) {
        assert_err!(Header::from_str(header));
    }

    #[test]
    fn test_invalid_content_length_header_empty() {
        invalid_header("Content-Length:");
    }

    #[test]
    fn test_invalid_content_length_header_empty_with_space_characters() {
        invalid_header("Content-Length:    ");
    }

    #[test]
    fn test_invalid_content_length_header_with_invalid_character_1() {
        invalid_header("Content-Length: 😁");
    }

    #[test]
    fn test_invalid_content_length_header_with_invalid_character_2() {
        invalid_header("Content-Length: mysize");
    }

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::ContentLength(first_header), Header::ContentLength(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not a Content-Length header");
        }
    }

    #[test]
    fn test_content_length_header_equality_same_header_with_space_characters_differences() {
        header_equality("Content-Length: 349", "Content-Length:     349");
    }

    #[test]
    fn test_content_length_header_equality_one_in_normal_form_the_other_in_compact_form() {
        header_equality("Content-Length: 283", "l: 283");
    }

    #[test]
    fn test_content_length_header_inequality() {
        let first_header = Header::from_str("Content-Length: 349");
        let second_header = Header::from_str("Content-Length: 173");
        if let (Header::ContentLength(first_header), Header::ContentLength(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not a Content-Length header");
        }
    }

    #[test]
    fn test_content_length_header_to_string() {
        let header = Header::from_str("cOntEnt-lEngth  :   349");
        if let Header::ContentLength(header) = header.unwrap() {
            assert_eq!(header.to_string(), "cOntEnt-lEngth  :   349");
            assert_eq!(header.to_normalized_string(), "Content-Length: 349");
            assert_eq!(header.to_compact_string(), "l: 349");
        }
    }
}
