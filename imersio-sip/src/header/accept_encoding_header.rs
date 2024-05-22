use std::{collections::HashSet, hash::Hash};

use crate::common::AcceptParameter;

#[derive(Clone, Debug)]
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

impl Eq for AcceptEncodingHeader {}

#[derive(Clone, Debug)]
pub struct Encoding {
    encoding: String,
    parameters: Vec<AcceptParameter>,
}

impl Encoding {
    pub(crate) fn new(encoding: String, parameters: Vec<AcceptParameter>) -> Self {
        Encoding {
            encoding,
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
        if self.encoding != other.encoding {
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

impl Eq for Encoding {}

impl Hash for Encoding {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.encoding.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::Header;
    use std::str::FromStr;

    #[test]
    fn test_valid_accept_encoding_header() {
        let header = Header::from_str("Accept-Encoding: gzip");
        assert!(header.is_ok());
        if let Header::AcceptEncoding(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 1);
            assert!(header.contains("gzip"));
            assert!(!header.contains("compress"));
            assert!(!header.contains("deflate"));
        } else {
            panic!("Not an Accept-Encoding header");
        }

        let header = Header::from_str("Accept-Encoding: gzip, deflate");
        assert!(header.is_ok());
        if let Header::AcceptEncoding(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 2);
            assert!(header.contains("gzip"));
            assert!(!header.contains("compress"));
            assert!(header.contains("deflate"));
        } else {
            panic!("Not an Accept-Encoding header");
        }

        let header = Header::from_str("Accept-Encoding: gzip    ,compress,  deflate");
        assert!(header.is_ok());
        if let Header::AcceptEncoding(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 3);
            assert!(header.contains("gzip"));
            assert!(header.contains("compress"));
            assert!(header.contains("deflate"));
        } else {
            panic!("Not an Accept-Encoding header");
        }

        // Empty Accept-Encoding header
        let header = Header::from_str("Accept-Encoding:");
        assert!(header.is_ok());
        if let Header::AcceptEncoding(header) = header.unwrap() {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains("gzip"));
            assert!(!header.contains("compress"));
            assert!(!header.contains("deflate"));
        } else {
            panic!("Not an Accept-Encoding header");
        }

        // Empty Accept-Encoding header with space characters
        let header = Header::from_str("Accept-Encoding:     ");
        assert!(header.is_ok());
        if let Header::AcceptEncoding(header) = header.unwrap() {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains("gzip"));
            assert!(!header.contains("compress"));
            assert!(!header.contains("deflate"));
        } else {
            panic!("Not an Accept-Encoding header");
        }

        // Accept-Encoding header with parameter
        let header = Header::from_str("Accept-Encoding: deflate, gzip;q=1.0");
        assert!(header.is_ok());
        if let Header::AcceptEncoding(header) = header.unwrap() {
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
        } else {
            panic!("Not an Accept-Encoding header");
        }
    }

    #[test]
    fn test_invalid_accept_encoding_header() {
        let header = Header::from_str("Accept-Encoding: 😁");
        assert!(header.is_err());
    }

    #[test]
    fn test_accept_encoding_header_equality() {
        let first_header = Header::from_str("Accept-Encoding: gzip");
        let second_header = Header::from_str("Accept-Encoding: gzip");
        if let (Header::AcceptEncoding(first_header), Header::AcceptEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Accept-Encoding header");
        }

        let first_header = Header::from_str("Accept-Encoding: gzip, deflate");
        let second_header = Header::from_str("Accept-Encoding: deflate, gzip");
        if let (Header::AcceptEncoding(first_header), Header::AcceptEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Accept-Encoding header");
        }
    }

    #[test]
    fn test_accept_encoding_header_inequality() {
        let first_header = Header::from_str("Accept-Encoding: gzip");
        let second_header = Header::from_str("Accept-Encoding: deflate");
        if let (Header::AcceptEncoding(first_header), Header::AcceptEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept-Encoding header");
        }

        let first_header = Header::from_str("Accept-Encoding: gzip, deflate");
        let second_header = Header::from_str("Accept-Encoding: deflate");
        if let (Header::AcceptEncoding(first_header), Header::AcceptEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept-Encoding header");
        }

        let first_header = Header::from_str("Accept-Encoding: deflate");
        let second_header = Header::from_str("Accept-Encoding: gzip, deflate");
        if let (Header::AcceptEncoding(first_header), Header::AcceptEncoding(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept-Encoding header");
        }
    }
}
