use partial_eq_refs::PartialEqRefs;

use super::{generic_header::GenericHeader, HeaderAccessor};

/// Representation of an Organization header.
///
/// The Organization header field conveys the name of the organization to which the SIP element
/// issuing the request or response belongs.
///
/// The field MAY be used by client software to filter calls.
///
/// [[RFC3261, Section 20.25](https://datatracker.ietf.org/doc/html/rfc3261#section-20.25)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct OrganizationHeader {
    header: GenericHeader,
    organization: Option<String>,
}

impl OrganizationHeader {
    pub(crate) fn new<S: Into<String>>(header: GenericHeader, organization: Option<S>) -> Self {
        Self {
            header,
            organization: organization.map(Into::into),
        }
    }

    /// Get the organization from the Organization header.
    pub fn organization(&self) -> Option<&str> {
        self.organization.as_deref()
    }
}

impl HeaderAccessor for OrganizationHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Organization")
    }
    fn normalized_value(&self) -> String {
        match &self.organization {
            Some(value) => value.clone(),
            None => "".to_string(),
        }
    }
}

impl std::fmt::Display for OrganizationHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for OrganizationHeader {
    fn eq(&self, other: &OrganizationHeader) -> bool {
        self.organization == other.organization
    }
}

#[cfg(test)]
mod tests {
    use super::OrganizationHeader;
    use crate::{
        header::{
            tests::{header_equality, header_inequality, valid_header},
            HeaderAccessor,
        },
        Header,
    };
    use claims::assert_ok;
    use std::str::FromStr;

    valid_header!(Organization, OrganizationHeader, "Organization");
    header_equality!(Organization, "Organization");
    header_inequality!(Organization, "Organization");

    #[test]
    fn test_valid_organization_header() {
        valid_header("Organization: Boxes by Bob", |header| {
            assert_eq!(header.organization(), Some("Boxes by Bob"));
        });
    }

    #[test]
    fn test_valid_organization_header_empty() {
        valid_header("Organization:", |header| {
            assert_eq!(header.organization(), None);
        });
    }

    #[test]
    fn test_valid_organization_header_empty_with_space_characters() {
        valid_header("Organization:    ", |header| {
            assert_eq!(header.organization(), None);
        });
    }

    #[test]
    fn test_valid_organization_header_with_utf8_character() {
        valid_header("Organization: 😁", |header| {
            assert_eq!(header.organization(), Some("😁"));
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
        let header = Header::from_str("organiZaTioN    :   Boxes by Bob");
        if let Header::Organization(header) = header.unwrap() {
            assert_eq!(header.to_string(), "organiZaTioN    :   Boxes by Bob");
            assert_eq!(header.to_normalized_string(), "Organization: Boxes by Bob");
            assert_eq!(header.to_compact_string(), "Organization: Boxes by Bob");
        }
    }
}
