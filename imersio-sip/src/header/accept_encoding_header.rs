use std::{collections::HashSet, hash::Hash};

use crate::{
    common::{AcceptParameter, HeaderValueCollection},
    utils::partial_eq_refs,
    HeaderAccessor,
};

use super::generic_header::GenericHeader;

/// Representation of an Accept-Encoding header.
///
/// The Accept-Encoding header field is similar to Accept, but restricts the
/// content-codings that are acceptable in the response.
///
/// [[RFC3261, Section 20.2](https://datatracker.ietf.org/doc/html/rfc3261#section-20.2)]
#[derive(Clone, Debug, Eq)]
pub struct AcceptEncodingHeader {
    header: GenericHeader,
    encodings: AcceptEncodings,
}

impl AcceptEncodingHeader {
    pub(crate) fn new(header: GenericHeader, encodings: Vec<AcceptEncoding>) -> Self {
        AcceptEncodingHeader {
            header,
            encodings: encodings.into(),
        }
    }

    /// Get a reference to the encodings of the `Accept-Encoding` header.
    pub fn encodings(&self) -> &AcceptEncodings {
        &self.encodings
    }
}

impl HeaderAccessor for AcceptEncodingHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Accept-Encoding")
    }
    fn normalized_value(&self) -> String {
        self.encodings.to_string()
    }
}

impl std::fmt::Display for AcceptEncodingHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for AcceptEncodingHeader {
    fn eq(&self, other: &Self) -> bool {
        self.encodings == other.encodings
    }
}

partial_eq_refs!(AcceptEncodingHeader);

/// Representation of the list of encodings from an `AcceptEncodingHeader`.
///
/// This is usable as an iterator.
pub type AcceptEncodings = HeaderValueCollection<AcceptEncoding>;

impl AcceptEncodings {
    /// Tell whether the encodings contain the given encoding.
    pub fn contains(&self, encoding: &str) -> bool {
        self.iter().any(|e| e.encoding == encoding)
    }

    /// Get the `Encoding` corresponding to the given encoding name.
    pub fn get(&self, encoding: &str) -> Option<&AcceptEncoding> {
        self.iter().find(|e| e.encoding == encoding)
    }
}

/// Representation of an encoding from an `Accept-Encoding` header.
#[derive(Clone, Debug, Eq)]
pub struct AcceptEncoding {
    encoding: String,
    parameters: Vec<AcceptParameter>,
}

impl AcceptEncoding {
    pub(crate) fn new<S: Into<String>>(encoding: S, parameters: Vec<AcceptParameter>) -> Self {
        AcceptEncoding {
            encoding: encoding.into(),
            parameters,
        }
    }

    /// Get the encoding.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// Get a reference to the parameters for the encoding.
    pub fn parameters(&self) -> &Vec<AcceptParameter> {
        &self.parameters
    }

    /// Get the value of the `q` parameter for the encoding, if it has one.
    pub fn q(&self) -> Option<f32> {
        self.parameters
            .iter()
            .find(|param| matches!(param, AcceptParameter::Q(_)))
            .and_then(|param| param.q())
    }
}

impl std::fmt::Display for AcceptEncoding {
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

impl PartialEq for AcceptEncoding {
    fn eq(&self, other: &Self) -> bool {
        if !self.encoding.eq_ignore_ascii_case(&other.encoding) {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

partial_eq_refs!(AcceptEncoding);

impl Hash for AcceptEncoding {
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
    use crate::{Header, HeaderAccessor};
    use claim::{assert_err, assert_ok};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(AcceptEncodingHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert_ok!(&header);
        if let Header::AcceptEncoding(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not an Accept-Encoding header");
        }
    }

    #[test]
    fn test_valid_accept_encoding_header_with_single_encoding() {
        valid_header("Accept-Encoding: gzip", |header| {
            assert!(!header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 1);
            assert!(header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(!header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_with_several_encodings() {
        valid_header("Accept-Encoding: gzip, deflate", |header| {
            assert!(!header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 2);
            assert!(header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_with_several_encodings_and_space_characters() {
        valid_header("Accept-Encoding: gzip    ,compress,  deflate", |header| {
            assert!(!header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 3);
            assert!(header.encodings().contains("gzip"));
            assert!(header.encodings().contains("compress"));
            assert!(header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_empty() {
        valid_header("Accept-Encoding:", |header| {
            assert!(header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 0);
            assert!(!header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(!header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_empty_with_space_characters() {
        valid_header("Accept-Encoding:     ", |header| {
            assert!(header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 0);
            assert!(!header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(!header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_with_parameter() {
        valid_header("Accept-Encoding: deflate, gzip;q=1.0", |header| {
            assert!(!header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 2);
            assert!(header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(header.encodings().contains("deflate"));
            let gzip_encoding = header.encodings().get("gzip").unwrap();
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
        assert_err!(header);
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

    #[test]
    fn test_accept_encoding_header_to_string() {
        let header = Header::from_str("accept-encoding:   gZip  , DeFlate");
        if let Header::Accept(header) = header.unwrap() {
            assert_eq!(header.to_string(), "accept-encoding:   gZip  , DeFlate");
            assert_eq!(
                header.to_normalized_string(),
                "Accept-Encoding: gzip, deflate"
            );
            assert_eq!(header.to_compact_string(), "Accept-Encoding: gzip, deflate");
        }
    }
}
