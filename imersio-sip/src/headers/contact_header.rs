//! SIP Contact header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::Contacts;

/// Representation of a Contact header.
///
/// A Contact header field value provides a URI whose meaning depends on the
/// type of request or response it is in.
/// The Contact header field has a role similar to the Location header field
/// in HTTP.
///
/// [[RFC3261, Section 20.10](https://datatracker.ietf.org/doc/html/rfc3261#section-20.10)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct ContactHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    contacts: Contacts,
}

impl ContactHeader {
    pub(crate) fn new(header: GenericHeader, contacts: Contacts) -> Self {
        Self { header, contacts }
    }

    /// Get a reference to the contacts from the Contact header.
    pub fn contacts(&self) -> &Contacts {
        &self.contacts
    }
}

impl HeaderAccessor for ContactHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("m")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Contact")
    }
    fn normalized_value(&self) -> String {
        self.contacts.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::contact::parser::contact_param;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, star, ParserResult};
    use crate::{ContactHeader, Contacts, Header};
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn contact(input: &str) -> ParserResult<&str, Header> {
        context(
            "Contact header",
            map(
                tuple((
                    alt((tag_no_case("Contact"), tag_no_case("m"))),
                    hcolon,
                    cut(consumed(alt((
                        map(star, |_| Contacts::Any),
                        map(separated_list1(comma, contact_param), Contacts::Contacts),
                    )))),
                )),
                |(name, separator, (value, contacts))| {
                    Header::Contact(ContactHeader::new(
                        GenericHeader::new(name, separator, value),
                        contacts,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            contact_header::Contacts,
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        ContactHeader, Header, Uri,
    };
    use claims::assert_ok;

    valid_header!(Contact, ContactHeader, "Contact");
    header_equality!(Contact, "Contact");
    header_inequality!(Contact, "Contact");

    #[test]
    fn test_valid_contact_header() {
        valid_header(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600, "Mr. Watson" <mailto:watson@bell-telephone.com> ;q=0.1"#,
            |header| {
                assert_eq!(header.contacts().len(), 2);
                let mut contacts = header.contacts().iter();
                let first_contact = contacts.next().unwrap();
                assert_eq!(first_contact.address().display_name(), Some("Mr. Watson"));
                assert_eq!(
                    first_contact.address().uri(),
                    Uri::try_from("sip:watson@worcester.bell-telephone.com").unwrap()
                );
                assert!(first_contact.address().uri().parameters().is_empty());
                assert_eq!(first_contact.parameters().len(), 2);
                assert!((first_contact.q().unwrap() - 0.7).abs() < 0.01);
                assert_eq!(first_contact.expires().unwrap(), 3600);
                let second_contact = contacts.next().unwrap();
                assert_eq!(second_contact.address().display_name(), Some("Mr. Watson"));
                assert_eq!(
                    second_contact.address().uri(),
                    Uri::try_from("mailto:watson@bell-telephone.com").unwrap()
                );
                assert!(second_contact.address().uri().parameters().is_empty());
                assert_eq!(second_contact.parameters().len(), 1);
                assert!((second_contact.q().unwrap() - 0.1).abs() < 0.01);
            },
        );
    }

    #[test]
    fn test_valid_contact_header_wildcard() {
        valid_header("Contact: *", |header| {
            assert_eq!(header.contacts(), &Contacts::Any);
        });
    }

    #[test]
    fn test_valid_contact_header_compact_wildcard() {
        valid_header("m: *", |header| {
            assert_eq!(header.contacts(), &Contacts::Any);
        });
    }

    #[test]
    fn test_invalid_contact_header_empty() {
        invalid_header("Contact:");
    }

    #[test]
    fn test_invalid_contact_header_empty_with_space_characters() {
        invalid_header("Contact:    ");
    }

    #[test]
    fn test_invalid_contact_header_with_invalid_character() {
        invalid_header("Contact: 😁");
    }

    #[test]
    fn test_contact_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com> ;q=0.7;expires=3600"#,
        );
    }

    #[test]
    fn test_contact_header_equality_contacts_in_a_different_order() {
        header_equality(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600, "Mr. Watson" <mailto:watson@bell-telephone.com> ;q=0.1"#,
            r#"Contact: "Mr. Watson" <mailto:watson@bell-telephone.com> ;q=0.1, "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
    }

    #[test]
    fn test_contact_header_equality_same_wildcard_with_space_characters_differences() {
        header_equality("Contact: *", "Contact:   *");
    }

    #[test]
    fn test_contact_header_equality_same_header_with_different_display_names() {
        header_equality(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
            r#"Contact: "Mrs. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
    }

    #[test]
    fn test_contact_header_equality_same_header_with_different_cases() {
        header_equality(
            "Contact: <sip:alice@atlanta.com>;expires=3600",
            "CONTACT: <sip:alice@atlanta.com>;ExPiReS=3600",
        );
    }

    #[test]
    fn test_contact_header_inequality_different_uris() {
        header_inequality(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
            r#"Contact: "Mr. Watson" <sip:watson@manchester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
    }

    #[test]
    fn test_contact_header_inequality_different_q_parameters() {
        header_inequality(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.5; expires=3600"#,
        );
    }

    #[test]
    fn test_contact_header_inequality_different_expires_parameters() {
        header_inequality(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3200"#,
        );
    }

    #[test]
    fn test_contact_header_to_string() {
        let header = Header::try_from(
            r#"contact: "Mr. Watson"  <sip:watson@worcester.bell-telephone.com>;Q=0.7; expIres=3600, "Mr. Watson" <mailto:watson@bell-telephone.com> ;  q=0.1"#,
        );
        if let Header::Contact(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"contact: "Mr. Watson"  <sip:watson@worcester.bell-telephone.com>;Q=0.7; expIres=3600, "Mr. Watson" <mailto:watson@bell-telephone.com> ;  q=0.1"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7;expires=3600, "Mr. Watson" <mailto:watson@bell-telephone.com>;q=0.1"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"m: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7;expires=3600, "Mr. Watson" <mailto:watson@bell-telephone.com>;q=0.1"#
            );
        }
    }
}
