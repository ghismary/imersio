use std::collections::HashSet;

#[derive(Clone, Debug, Eq)]
pub struct ContentEncodingHeader(Vec<String>);

impl ContentEncodingHeader {
    pub(crate) fn new<S: Into<String>>(encodings: Vec<S>) -> Self {
        ContentEncodingHeader(
            encodings
                .into_iter()
                .map(Into::into)
                .collect::<Vec<String>>(),
        )
    }

    /// Get a reference to the encodings from the Content-Encoding header.
    pub fn encodings(&self) -> &Vec<String> {
        &self.0
    }
}

impl std::fmt::Display for ContentEncodingHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Content-Encoding: {}", self.0.join(", "))
    }
}

impl PartialEq<ContentEncodingHeader> for ContentEncodingHeader {
    fn eq(&self, other: &ContentEncodingHeader) -> bool {
        let self_encodings: HashSet<_> = self.0.iter().map(|v| v.to_ascii_lowercase()).collect();
        let other_encodings: HashSet<_> = other.0.iter().map(|v| v.to_ascii_lowercase()).collect();
        self_encodings == other_encodings
    }
}

impl PartialEq<&ContentEncodingHeader> for ContentEncodingHeader {
    fn eq(&self, other: &&ContentEncodingHeader) -> bool {
        self == *other
    }
}

impl PartialEq<ContentEncodingHeader> for &ContentEncodingHeader {
    fn eq(&self, other: &ContentEncodingHeader) -> bool {
        *self == other
    }
}

#[cfg(test)]
mod tests {
    use super::ContentEncodingHeader;
    use crate::Header;
    use std::str::FromStr;

    fn valid_header<F: FnOnce(ContentEncodingHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert!(header.is_ok());
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
        assert!(Header::from_str(header).is_err());
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
}
