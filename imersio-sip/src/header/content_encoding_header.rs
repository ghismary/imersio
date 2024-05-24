use std::collections::HashSet;

#[derive(Clone, Debug)]
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

impl Eq for ContentEncodingHeader {}

#[cfg(test)]
mod tests {
    use crate::Header;
    use std::str::FromStr;

    #[test]
    fn test_valid_content_encoding_header() {
        // Valid Content-Encoding header.
        let header = Header::from_str("Content-Encoding: gzip");
        assert!(header.is_ok());
        if let Header::ContentEncoding(header) = header.unwrap() {
            assert_eq!(header.encodings().len(), 1);
            assert_eq!(header.encodings().first().unwrap(), "gzip");
        } else {
            panic!("Not a Content-Encoding header");
        }

        // Valid Content-Encoding header in its compact form.
        let header = Header::from_str("e: tar");
        assert!(header.is_ok());
        if let Header::ContentEncoding(header) = header.unwrap() {
            assert_eq!(header.encodings().len(), 1);
            assert_eq!(header.encodings().first().unwrap(), "tar");
        } else {
            panic!("Not a Content-Encoding header");
        }
    }

    #[test]
    fn test_invalid_content_encoding_header() {
        // Empty Content-Encoding header.
        let header = Header::from_str("Content-Encoding:");
        assert!(header.is_err());

        // Empty Content-Encoding header with spaces.
        let header = Header::from_str("Content-Encoding:    ");
        assert!(header.is_err());

        // Content-Encoding header with invalid character.
        let header = Header::from_str("Content-Encoding: 😁");
        assert!(header.is_err());
    }

    #[test]
    fn test_content_encoding_header_equality() {
        // Same Content-Encoding header, with just some space characters differences.
        let first_header = Header::from_str("Content-Encoding: gzip");
        let second_header = Header::from_str("Content-Encoding:  gzip");
        if let (Header::ContentEncoding(first_header), Header::ContentEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not a Content-Encoding header");
        }

        // Content-Encoding header with the same encodings in a different order.
        let first_header = Header::from_str("Content-Encoding: gzip, tar");
        let second_header = Header::from_str("Content-Encoding: tar, gzip");
        if let (Header::ContentEncoding(first_header), Header::ContentEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not a Content-Encoding header");
        }

        // Content-Encoding header with the same encodings with different cases.
        let first_header = Header::from_str("Content-Encoding: gzip");
        let second_header = Header::from_str("content-encoding: GZIP");
        if let (Header::ContentEncoding(first_header), Header::ContentEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not a Content-Encoding header");
        }
    }

    #[test]
    fn test_content_encoding_header_inequality() {
        // Obviously different Content-Encoding headers.
        let first_header = Header::from_str("Content-Encoding: gzip");
        let second_header = Header::from_str("Content-Encoding: tar");
        if let (Header::ContentEncoding(first_header), Header::ContentEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not a Content-Encoding header");
        }

        let first_header = Header::from_str("Content-Encoding: gzip, tar");
        let second_header = Header::from_str("Content-Encoding: tar");
        if let (Header::ContentEncoding(first_header), Header::ContentEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not a Content-Encoding header");
        }

        let first_header = Header::from_str("Content-Encoding: gzip");
        let second_header = Header::from_str("Content-Encoding: tar, gzip");
        if let (Header::ContentEncoding(first_header), Header::ContentEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not a Content-Encoding header");
        }
    }
}
