use partial_eq_refs::PartialEqRefs;

use super::{generic_header::GenericHeader, HeaderAccessor};

/// Representation of an Expires header.
///
/// The Expires header field gives the relative time after which the message (or content) expires.
/// The precise meaning of this is method dependent.
/// The expiration time in an INVITE does not affect the duration of the actual session that may
/// result from the invitation. Session description protocols may offer the ability to express time
/// limits on the session duration, however.
///
/// [[RFC3261, Section 20.19](https://datatracker.ietf.org/doc/html/rfc3261#section-20.19)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ExpiresHeader {
    header: GenericHeader,
    expires: u32,
}

impl ExpiresHeader {
    pub(crate) fn new(header: GenericHeader, expires: u32) -> Self {
        Self { header, expires }
    }

    /// Get the expires from the Expires header.
    pub fn expires(&self) -> u32 {
        self.expires
    }
}

impl HeaderAccessor for ExpiresHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Expires")
    }
    fn normalized_value(&self) -> String {
        self.expires.to_string()
    }
}

impl std::fmt::Display for ExpiresHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for ExpiresHeader {
    fn eq(&self, other: &ExpiresHeader) -> bool {
        self.expires == other.expires
    }
}

#[cfg(test)]
mod tests {
    use super::ExpiresHeader;
    use crate::{
        header::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header,
    };
    use claims::assert_ok;
    use std::str::FromStr;

    valid_header!(Expires, ExpiresHeader, "Expires");
    header_equality!(Expires, "Expires");
    header_inequality!(Expires, "Expires");

    #[test]
    fn test_valid_expires_header() {
        valid_header("Expires: 5", |header| {
            assert_eq!(header.expires(), 5);
        });
    }

    #[test]
    fn test_valid_expires_header_with_value_too_big() {
        valid_header("Expires: 4294968000", |header| {
            assert_eq!(header.expires(), u32::MAX);
        });
    }

    #[test]
    fn test_invalid_expires_header_empty() {
        invalid_header("Expires:");
    }

    #[test]
    fn test_invalid_expires_header_empty_with_space_characters() {
        invalid_header("Expires:    ");
    }

    #[test]
    fn test_invalid_expires_header_with_invalid_character() {
        invalid_header("Expires: 😁");
    }

    #[test]
    fn test_expires_header_equality_same_header_with_space_characters_differences() {
        header_equality("Expires: 3600", "Expires :   3600");
    }

    #[test]
    fn test_expires_header_inequality_different_values() {
        header_inequality("Expires: 3600", "Expires: 5");
    }

    #[test]
    fn test_expires_header_to_string() {
        let header = Header::from_str("eXpires  :     3600");
        if let Header::Expires(header) = header.unwrap() {
            assert_eq!(header.to_string(), "eXpires  :     3600");
            assert_eq!(header.to_normalized_string(), "Expires: 3600");
            assert_eq!(header.to_compact_string(), "Expires: 3600");
        }
    }
}
