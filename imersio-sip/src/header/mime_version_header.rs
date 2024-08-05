//! SIP MIME-Version header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::{header::GenericHeader, HeaderAccessor};

/// Representation of a MIME-Version header.
///
/// [[RFC3261, Section 20.24](https://datatracker.ietf.org/doc/html/rfc3261#section-20.24)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct MimeVersionHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    version: String,
}

impl MimeVersionHeader {
    pub(crate) fn new<S: Into<String>>(header: GenericHeader, version: S) -> Self {
        Self {
            header,
            version: version.into(),
        }
    }

    /// Get the version from the MIME-Version header.
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl HeaderAccessor for MimeVersionHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("MIME-Version")
    }
    fn normalized_value(&self) -> String {
        self.version.clone()
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_ok;

    use super::MimeVersionHeader;
    use crate::{
        header::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header,
    };

    valid_header!(MimeVersion, MimeVersionHeader, "MIME-Version");
    header_equality!(MimeVersion, "MIME-Version");
    header_inequality!(MimeVersion, "MIME-Version");

    #[test]
    fn test_valid_mime_version_header() {
        valid_header("MIME-Version: 1.0", |header| {
            assert_eq!(header.version(), "1.0");
        });
    }

    #[test]
    fn test_invalid_mime_version_header_empty() {
        invalid_header("MIME-Version:");
    }

    #[test]
    fn test_invalid_mime_version_header_empty_with_space_characters() {
        invalid_header("MIME-Version:    ");
    }

    #[test]
    fn test_invalid_mime_version_header_with_invalid_character() {
        invalid_header("MIME-Version: 😁");
    }

    #[test]
    fn test_invalid_mime_version_header_no_digit_before_dot() {
        invalid_header("MIME-Version: .0");
    }

    #[test]
    fn test_invalid_mime_version_header_no_digit_after_dot() {
        invalid_header("MIME-Version: 1.");
    }

    #[test]
    fn test_invalid_mime_version_header_digits_but_no_dot() {
        invalid_header("MIME-Version: 10");
    }

    #[test]
    fn test_mime_version_header_equality_same_header_with_space_characters_differences() {
        header_equality("MIME-Version: 1.0", "MIME-Version :     1.0");
    }

    #[test]
    fn test_mime_version_header_inequality_different_values() {
        header_inequality("MIME-Version: 1.0", "MIME-Version: 2.1");
    }

    #[test]
    fn test_mime_version_header_to_string() {
        let header = Header::try_from("mime-Version  :     1.0");
        if let Header::MimeVersion(header) = header.unwrap() {
            assert_eq!(header.to_string(), "mime-Version  :     1.0");
            assert_eq!(header.to_normalized_string(), "MIME-Version: 1.0");
            assert_eq!(header.to_compact_string(), "MIME-Version: 1.0");
        }
    }
}
