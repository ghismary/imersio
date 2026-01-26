//! SIP Organization header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};

/// Representation of an Organization header.
///
/// The Organization header field conveys the name of the organization to which the SIP element
/// issuing the request or response belongs.
///
/// The field MAY be used by client software to filter calls.
///
/// [[RFC3261, Section 20.25](https://datatracker.ietf.org/doc/html/rfc3261#section-20.25)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
pub struct OrganizationHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    organization: String,
}

impl OrganizationHeader {
    pub(crate) fn new<S: Into<String>>(header: GenericHeader, organization: S) -> Self {
        Self {
            header,
            organization: organization.into(),
        }
    }

    /// Get the organization from the Organization header.
    pub fn organization(&self) -> &str {
        &self.organization
    }
}

impl HeaderAccessor for OrganizationHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Organization")
    }
    fn normalized_value(&self) -> String {
        self.organization.clone()
    }
}

pub(crate) mod parser {
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map, opt},
        error::context,
        Parser,
    };

    use crate::{
        headers::GenericHeader,
        parser::{hcolon, text_utf8_trim, ParserResult},
        Header, OrganizationHeader, TokenString,
    };

    pub(crate) fn organization(input: &str) -> ParserResult<&str, Header> {
        context(
            "Organization header",
            map(
                (
                    map(tag_no_case("Organization"), TokenString::new),
                    hcolon,
                    cut(consumed(opt(text_utf8_trim))),
                ),
                |(name, separator, (value, organization))| {
                    Header::Organization(OrganizationHeader::new(
                        GenericHeader::new(name, separator, value),
                        organization.unwrap_or_default(),
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
            tests::{header_equality, header_inequality, valid_header},
            HeaderAccessor,
        },
        Header, OrganizationHeader,
    };
    use claims::assert_ok;

    valid_header!(Organization, OrganizationHeader, "Organization");
    header_equality!(Organization, "Organization");
    header_inequality!(Organization, "Organization");

    #[test]
    fn test_valid_organization_header() {
        valid_header("Organization: Boxes by Bob", |header| {
            assert_eq!(header.organization(), "Boxes by Bob");
        });
    }

    #[test]
    fn test_valid_organization_header_empty() {
        valid_header("Organization:", |header| {
            assert_eq!(header.organization(), "");
        });
    }

    #[test]
    fn test_valid_organization_header_empty_with_space_characters() {
        valid_header("Organization:    ", |header| {
            assert_eq!(header.organization(), "");
        });
    }

    #[test]
    fn test_valid_organization_header_with_utf8_character() {
        valid_header("Organization: 😁", |header| {
            assert_eq!(header.organization(), "😁");
        });
    }

    #[test]
    fn test_organization_header_equality_same_header_with_trailing_space_characters_differences() {
        header_equality(
            "Organization: Boxes by Bob",
            "Organization: Boxes by Bob    ",
        );
    }

    #[test]
    fn test_organization_header_inequality_different_values() {
        header_inequality("Organization: Boxes by Bob", "Organization: Axes by Alice");
    }

    #[test]
    fn test_organization_header_to_string() {
        let header = Header::try_from("organiZaTioN    :   Boxes by Bob");
        if let Header::Organization(header) = header.unwrap() {
            assert_eq!(header.to_string(), "organiZaTioN    :   Boxes by Bob");
            assert_eq!(header.to_normalized_string(), "Organization: Boxes by Bob");
            assert_eq!(header.to_compact_string(), "Organization: Boxes by Bob");
        }
    }
}
