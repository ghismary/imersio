use std::{collections::HashSet, hash::Hash};

use crate::common::AcceptParameter;

#[derive(Clone, Debug, Eq)]
pub struct AcceptEncodingHeader(Vec<Encoding>);

impl AcceptEncodingHeader {
    pub(crate) fn new(encodings: Vec<Encoding>) -> Self {
        AcceptEncodingHeader(encodings)
    }

    /// Tells whether the Accept-Encoding header is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of encodings in the Accept-Encoding header.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Tells whether Accept-Encoding header contains the given encoding.
    pub fn contains(&self, encoding: &str) -> bool {
        self.0.iter().any(|e| e.encoding == encoding)
    }

    /// Gets the `Encoding` corresponding to the given encoding name.
    pub fn get(&self, encoding: &str) -> Option<&Encoding> {
        self.0.iter().find(|e| e.encoding == encoding)
    }
}

impl std::fmt::Display for AcceptEncodingHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Accept-Encoding: {}",
            self.0
                .iter()
                .map(|encoding| encoding.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl PartialEq for AcceptEncodingHeader {
    fn eq(&self, other: &Self) -> bool {
        let self_encodings: HashSet<_> = self.0.iter().collect();
        let other_encodings: HashSet<_> = other.0.iter().collect();
        self_encodings == other_encodings
    }
}

impl PartialEq<&AcceptEncodingHeader> for AcceptEncodingHeader {
    fn eq(&self, other: &&AcceptEncodingHeader) -> bool {
        self == *other
    }
}

impl PartialEq<AcceptEncodingHeader> for &AcceptEncodingHeader {
    fn eq(&self, other: &AcceptEncodingHeader) -> bool {
        *self == other
    }
}

#[derive(Clone, Debug, Eq)]
pub struct Encoding {
    encoding: String,
    parameters: Vec<AcceptParameter>,
}

impl Encoding {
    pub(crate) fn new<S: Into<String>>(encoding: S, parameters: Vec<AcceptParameter>) -> Self {
        Encoding {
            encoding: encoding.into(),
            parameters,
        }
    }

    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    pub fn parameters(&self) -> &Vec<AcceptParameter> {
        &self.parameters
    }

    pub fn q(&self) -> Option<f32> {
        self.parameters
            .iter()
            .find(|param| matches!(param, AcceptParameter::Q(_)))
            .and_then(|param| param.q())
    }
}

impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.encoding,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl PartialEq for Encoding {
    fn eq(&self, other: &Self) -> bool {
        if !self.encoding.eq_ignore_ascii_case(&other.encoding) {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

impl PartialEq<&Encoding> for Encoding {
    fn eq(&self, other: &&Encoding) -> bool {
        self == *other
    }
}

impl PartialEq<Encoding> for &Encoding {
    fn eq(&self, other: &Encoding) -> bool {
        *self == other
    }
}

impl Hash for Encoding {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.encoding.to_ascii_lowercase().hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::AcceptEncodingHeader;
    use crate::Header;
    use std::str::FromStr;

    fn valid_header<F: FnOnce(AcceptEncodingHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert!(header.is_ok());
        if let Header::AcceptEncoding(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not an Accept-Encoding header");
        }
    }

    #[test]
    fn test_valid_accept_encoding_header_with_single_encoding() {
        valid_header("Accept-Encoding: gzip", |header| {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 1);
            assert!(header.contains("gzip"));
            assert!(!header.contains("compress"));
            assert!(!header.contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_with_several_encodings() {
        valid_header("Accept-Encoding: gzip, deflate", |header| {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 2);
            assert!(header.contains("gzip"));
            assert!(!header.contains("compress"));
            assert!(header.contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_with_several_encodings_and_space_characters() {
        valid_header("Accept-Encoding: gzip    ,compress,  deflate", |header| {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 3);
            assert!(header.contains("gzip"));
            assert!(header.contains("compress"));
            assert!(header.contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_empty() {
        valid_header("Accept-Encoding:", |header| {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains("gzip"));
            assert!(!header.contains("compress"));
            assert!(!header.contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_empty_with_space_characters() {
        valid_header("Accept-Encoding:     ", |header| {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains("gzip"));
            assert!(!header.contains("compress"));
            assert!(!header.contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_with_parameter() {
        valid_header("Accept-Encoding: deflate, gzip;q=1.0", |header| {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 2);
            assert!(header.contains("gzip"));
            assert!(!header.contains("compress"));
            assert!(header.contains("deflate"));
            let gzip_encoding = header.get("gzip").unwrap();
            assert_eq!(gzip_encoding.parameters().len(), 1);
            assert_eq!(gzip_encoding.parameters().first().unwrap().key(), "q");
            assert_eq!(
                gzip_encoding.parameters().first().unwrap().value(),
                Some("1.0")
            );
            let gzip_q = gzip_encoding.q();
            assert!(gzip_q.is_some());
            assert!((gzip_q.unwrap() - 1.0).abs() < 0.01);
        });
    }

    #[test]
    fn test_invalid_accept_encoding_header_with_invalid_character() {
        let header = Header::from_str("Accept-Encoding: 😁");
        assert!(header.is_err());
    }

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::AcceptEncoding(first_header), Header::AcceptEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Accept-Encoding header");
        }
    }

    #[test]
    fn test_accept_encoding_header_equality_with_space_characters_differences() {
        header_equality("Accept-Encoding: gzip", "Accept-Encoding:  gzip");
    }

    #[test]
    fn test_accept_encoding_header_equality_with_different_encodings_order() {
        header_equality(
            "Accept-Encoding: gzip, deflate",
            "Accept-Encoding: deflate, gzip",
        );
    }

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::AcceptEncoding(first_header), Header::AcceptEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept-Encoding header");
        }
    }

    #[test]
    fn test_accept_encoding_header_inequality_with_different_encodings() {
        header_inequality("Accept-Encoding: gzip", "Accept-Encoding: deflate");
    }

    #[test]
    fn test_accept_encoding_header_inequality_with_first_header_having_more_encodings_than_the_second(
    ) {
        header_inequality("Accept-Encoding: gzip, deflate", "Accept-Encoding: deflate");
    }

    #[test]
    fn test_accept_encoding_header_inequality_with_first_header_having_less_encodings_than_the_second(
    ) {
        header_inequality("Accept-Encoding: deflate", "Accept-Encoding: gzip, deflate");
    }
}
