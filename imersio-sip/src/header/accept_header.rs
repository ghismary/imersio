use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::GenericParameter;

#[derive(Clone, Debug, Default)]
pub struct AcceptHeader(Vec<AcceptRange>);

impl AcceptHeader {
    pub(crate) fn new(ranges: Vec<AcceptRange>) -> Self {
        AcceptHeader(ranges)
    }

    /// Tells whether the Accept header is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of `AcceptRange` in the Accept header.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Tells whether the Accept header contains the given `MediaRange`.
    pub fn contains(&self, media_range: &MediaRange) -> bool {
        self.0.iter().any(|ar| ar.media_range == media_range)
    }

    /// Gets the `Accept-Range` corresponding to the given `MediaRange`.
    pub fn get(&self, media_range: &MediaRange) -> Option<&AcceptRange> {
        self.0.iter().find(|ar| ar.media_range == media_range)
    }
}

impl std::fmt::Display for AcceptHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Accept: {}",
            self.0
                .iter()
                .map(|range| range.to_string())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

impl PartialEq for AcceptHeader {
    fn eq(&self, other: &Self) -> bool {
        let self_accept_ranges: HashSet<_> = self.0.iter().collect();
        let other_accept_ranges: HashSet<_> = other.0.iter().collect();
        self_accept_ranges == other_accept_ranges
    }
}

impl PartialEq<&AcceptHeader> for AcceptHeader {
    fn eq(&self, other: &&AcceptHeader) -> bool {
        self == *other
    }
}

impl PartialEq<AcceptHeader> for &AcceptHeader {
    fn eq(&self, other: &AcceptHeader) -> bool {
        *self == other
    }
}

impl Eq for AcceptHeader {}

#[derive(Clone, Debug)]
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
            if self.parameters.is_empty() { "" } else { "; " },
            self.parameters
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join("; ")
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

impl PartialEq<&AcceptRange> for AcceptRange {
    fn eq(&self, other: &&AcceptRange) -> bool {
        self == *other
    }
}

impl PartialEq<AcceptRange> for &AcceptRange {
    fn eq(&self, other: &AcceptRange) -> bool {
        *self == other
    }
}

impl Eq for AcceptRange {}

impl Hash for AcceptRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.media_range.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AcceptParameter {
    pub(crate) key: String,
    pub(crate) value: Option<String>,
}

impl AcceptParameter {
    pub(crate) fn new(key: String, value: Option<String>) -> Self {
        AcceptParameter { key, value }
    }
}

impl std::fmt::Display for AcceptParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.key,
            match &self.value {
                Some(value) => format!("; {}", value),
                None => "".to_string(),
            }
        )
    }
}

impl PartialEq<&AcceptParameter> for AcceptParameter {
    fn eq(&self, other: &&AcceptParameter) -> bool {
        self == *other
    }
}

impl PartialEq<AcceptParameter> for &AcceptParameter {
    fn eq(&self, other: &AcceptParameter) -> bool {
        *self == other
    }
}

impl PartialOrd for AcceptParameter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AcceptParameter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.key.cmp(&other.key) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value.cmp(&other.value)
    }
}

impl From<GenericParameter> for AcceptParameter {
    fn from(value: GenericParameter) -> Self {
        Self {
            key: value.key().to_string(),
            value: value.value().map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct MediaRange {
    r#type: String,
    subtype: String,
}

impl MediaRange {
    pub(crate) fn new(r#type: String, subtype: String) -> Self {
        MediaRange { r#type, subtype }
    }
}

impl std::fmt::Display for MediaRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.r#type, self.subtype,)
    }
}

impl PartialEq<&MediaRange> for MediaRange {
    fn eq(&self, other: &&MediaRange) -> bool {
        self == *other
    }
}

impl PartialEq<MediaRange> for &MediaRange {
    fn eq(&self, other: &MediaRange) -> bool {
        *self == other
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        header::accept_header::{AcceptParameter, MediaRange},
        Header,
    };
    use std::str::FromStr;

    #[test]
    fn test_valid_accept_header() {
        let header = Header::from_str("Accept: application/sdp");
        assert!(header.is_ok());
        if let Header::Accept(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 1);
            assert!(header.contains(&MediaRange::new(
                "application".to_string(),
                "sdp".to_string()
            )));
            assert!(!header.contains(&MediaRange::new(
                "application".to_string(),
                "x-private".to_string()
            )));
            assert!(!header.contains(&MediaRange::new("text".to_string(), "html".to_string())));
        } else {
            panic!("Not an Accept header");
        }

        let header =
            Header::from_str("Accept: application/sdp;level=1, application/x-private, text/html");
        assert!(header.is_ok());
        if let Header::Accept(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 3);
            assert!(header.contains(&MediaRange::new(
                "application".to_string(),
                "sdp".to_string()
            )));
            assert!(header.contains(&MediaRange::new(
                "application".to_string(),
                "x-private".to_string()
            )));
            assert!(header.contains(&MediaRange::new("text".to_string(), "html".to_string())));
            let accept_range = header.get(&MediaRange::new(
                "application".to_string(),
                "sdp".to_string(),
            ));
            assert!(accept_range.is_some());
            let accept_range = accept_range.unwrap();
            assert_eq!(accept_range.parameters().len(), 1);
            assert_eq!(
                accept_range.parameters().first().unwrap(),
                AcceptParameter::new("level".to_string(), Some("1".to_string()))
            );
            let accept_range = header.get(&MediaRange::new("text".to_string(), "html".to_string()));
            assert!(accept_range.is_some());
            let accept_range = accept_range.unwrap();
            assert!(accept_range.parameters().is_empty());
        } else {
            panic!("Not an Accept header");
        }

        let header = Header::from_str("Accept: */*");
        assert!(header.is_ok());
        if let Header::Accept(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 1);
            assert!(header.contains(&MediaRange::new("*".to_string(), "*".to_string())));
        } else {
            panic!("Not an Accept header");
        }

        let header = Header::from_str("Accept: text/*");
        assert!(header.is_ok());
        if let Header::Accept(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 1);
            assert!(header.contains(&MediaRange::new("text".to_string(), "*".to_string())));
        } else {
            panic!("Not an Accept header");
        }

        let header = Header::from_str("Accept:");
        assert!(header.is_ok());
        if let Header::Accept(header) = header.unwrap() {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains(&MediaRange::new(
                "application".to_string(),
                "sdp".to_string()
            )));
            assert!(!header.contains(&MediaRange::new("text".to_string(), "html".to_string())));
        } else {
            panic!("Not an Accept header");
        }

        let header = Header::from_str("Accept:  ");
        assert!(header.is_ok());
        if let Header::Accept(header) = header.unwrap() {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains(&MediaRange::new(
                "application".to_string(),
                "sdp".to_string()
            )));
            assert!(!header.contains(&MediaRange::new("text".to_string(), "html".to_string())));
        } else {
            panic!("Not an Accept header");
        }
    }

    #[test]
    fn test_invalid_accept_header() {
        let header = Header::from_str("Accept: application");
        assert!(header.is_err());

        let header = Header::from_str("Accept: application/");
        assert!(header.is_err());

        let header = Header::from_str("Accept: 😁/😁");
        assert!(header.is_err());
    }

    #[test]
    fn test_accept_header_equality() {
        let first_header = Header::from_str("Accept: text/html");
        let second_header = Header::from_str("Accept: text/html");
        if let (Header::Accept(first_header), Header::Accept(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Accept header");
        }

        let first_header = Header::from_str("Accept: text/html, application/sdp");
        let second_header = Header::from_str("Accept: application/sdp, text/html");
        if let (Header::Accept(first_header), Header::Accept(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Accept header");
        }
    }

    #[test]
    fn test_accept_header_inequality() {
        let first_header = Header::from_str("Accept: application/sdp");
        let second_header = Header::from_str("Accept: text/html");
        if let (Header::Accept(first_header), Header::Accept(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept header");
        }

        let first_header = Header::from_str("Accept: application/sdp, text/html");
        let second_header = Header::from_str("Accept: text/html");
        if let (Header::Accept(first_header), Header::Accept(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept header");
        }

        let first_header = Header::from_str("Accept: text/html");
        let second_header = Header::from_str("Accept: application/sdp, text/html");
        if let (Header::Accept(first_header), Header::Accept(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept header");
        }
    }
}
