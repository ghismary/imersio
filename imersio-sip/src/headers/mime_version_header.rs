//! SIP MIME-Version header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};

/// Representation of a MIME-Version header.
///
/// [[RFC3261, Section 20.24](https://datatracker.ietf.org/doc/html/rfc3261#section-20.24)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
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
    crate::headers::generic_header_accessors!(header);

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

pub(crate) mod parser {
    use nom::{
        bytes::complete::{tag, tag_no_case},
        combinator::{consumed, cut, map, recognize},
        error::context,
        multi::many1,
        Parser,
    };

    use crate::{
        headers::GenericHeader,
        parser::{digit, hcolon, ParserResult},
        Header, MimeVersionHeader, TokenString,
    };

    pub(crate) fn mime_version(input: &str) -> ParserResult<&str, Header> {
        context(
            "MIME-Version header",
            map(
                (
                    map(tag_no_case("MIME-Version"), TokenString::new),
                    hcolon,
                    cut(consumed(recognize((many1(digit), tag("."), many1(digit))))),
                ),
                |(name, separator, (value, version))| {
                    Header::MimeVersion(MimeVersionHeader::new(
                        GenericHeader::new(name, separator, value),
                        version,
                    ))
                },
            ),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, MimeVersionHeader,
    };
    use claims::assert_ok;

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
