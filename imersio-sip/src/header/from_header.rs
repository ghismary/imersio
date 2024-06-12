use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use super::generic_header::GenericHeader;
use crate::common::from_parameter::{FromParameter, FromParameters};
use crate::{common::name_address::NameAddress, HeaderAccessor};

/// Representation of a From header.
///
/// The From header field indicates the initiator of the request. This may be different from the
/// initiator of the dialog. Requests sent by the callee to the caller use the callee's address in
/// the From header field.
///
/// [[RFC3261, Section 20.20](https://datatracker.ietf.org/doc/html/rfc3261#section-20.20)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct FromHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    address: NameAddress,
    parameters: FromParameters,
}

impl FromHeader {
    pub(crate) fn new(
        header: GenericHeader,
        address: NameAddress,
        parameters: Vec<FromParameter>,
    ) -> Self {
        Self {
            header,
            address,
            parameters: parameters.into(),
        }
    }

    /// Get a reference to the address from the From header.
    pub fn address(&self) -> &NameAddress {
        &self.address
    }

    /// Get a reference to the parameters from the From header.
    pub fn parameters(&self) -> &FromParameters {
        &self.parameters
    }

    /// Get the value of the `tag` parameter from the From header, if it has one.
    pub fn tag(&self) -> Option<&str> {
        self.parameters
            .iter()
            .find(|param| matches!(param, FromParameter::Tag(_)))
            .and_then(|param| param.tag())
    }
}

impl HeaderAccessor for FromHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("f")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("From")
    }
    fn normalized_value(&self) -> String {
        format!(
            "{}{}{}",
            self.address,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
        )
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_ok;

    use super::FromHeader;
    use crate::{
        header::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, Uri,
    };

    valid_header!(From, FromHeader, "From");
    header_equality!(From, "From");
    header_inequality!(From, "From");

    #[test]
    fn test_valid_from_header_with_display_name() {
        valid_header(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            |header| {
                assert_eq!(header.address().display_name(), Some("A. G. Bell"));
                assert_eq!(
                    header.address().uri(),
                    Uri::try_from("sip:agb@bell-telephone.com").unwrap()
                );
                assert_eq!(header.parameters().len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("a48s"));
                assert_eq!(header.tag(), Some("a48s"));
            },
        );
    }

    #[test]
    fn test_valid_from_header_without_display_name() {
        valid_header(
            "From: <sip:+12125551212@server.phone2net.com>;tag=887s",
            |header| {
                assert_eq!(header.address().display_name(), None);
                assert_eq!(
                    header.address().uri(),
                    Uri::try_from("sip:+12125551212@server.phone2net.com").unwrap()
                );
                assert_eq!(header.parameters.len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("887s"));
                assert_eq!(header.tag(), Some("887s"));
            },
        )
    }

    #[test]
    fn test_valid_from_header_in_compact_form() {
        valid_header(
            "f: Anonymous <sip:c8oqz84zk7z@privacy.org>;tag=hyh8",
            |header| {
                assert_eq!(header.address().display_name(), Some("Anonymous"));
                assert_eq!(
                    header.address().uri(),
                    Uri::try_from("sip:c8oqz84zk7z@privacy.org").unwrap()
                );
                assert_eq!(header.parameters.len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("hyh8"));
                assert_eq!(header.tag(), Some("hyh8"));
            },
        )
    }

    #[test]
    fn test_invalid_from_header_empty() {
        invalid_header("From:");
    }

    #[test]
    fn test_invalid_from_header_empty_with_space_characters() {
        invalid_header("From:    ");
    }

    #[test]
    fn test_invalid_from_header_with_invalid_character() {
        invalid_header("From: 😁");
    }

    #[test]
    fn test_from_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From:    "A. G. Bell"  <sip:agb@bell-telephone.com>; tag=a48s"#,
        );
    }

    #[test]
    fn test_from_header_equality_same_header_with_different_display_names() {
        header_equality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: Bell <sip:agb@bell-telephone.com> ;tag=a48s"#,
        );
    }

    #[test]
    fn test_from_header_equality_same_header_with_different_cases() {
        header_equality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;TAG=a48s"#,
        );
    }

    #[test]
    fn test_from_header_inequality_different_uris() {
        header_inequality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: "A. G. Bell" <sip:agc@bell-telephone.com> ;tag=a48s"#,
        );
    }

    #[test]
    fn test_from_header_inequality_different_tag_parameters() {
        header_inequality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=hyh8"#,
        );
    }

    #[test]
    fn test_from_header_to_string() {
        let header = Header::try_from(
            r#"from :    "A. G. Bell"   <sip:agb@bell-telephone.com> ;   tag  = a48s"#,
        );
        if let Header::From(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"from :    "A. G. Bell"   <sip:agb@bell-telephone.com> ;   tag  = a48s"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"From: "A. G. Bell" <sip:agb@bell-telephone.com>;tag=a48s"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"f: "A. G. Bell" <sip:agb@bell-telephone.com>;tag=a48s"#
            );
        }
    }
}
