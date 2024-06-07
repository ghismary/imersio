use std::{cmp::Ordering, collections::HashSet, hash::Hash, ops::Deref};

use partial_eq_refs::PartialEqRefs;

use crate::{common::name_address::NameAddress, GenericParameter, HeaderAccessor};

use super::generic_header::GenericHeader;

static EMPTY_CONTACTS: Vec<Contact> = vec![];

/// Representation of a Contact header.
///
/// A Contact header field value provides a URI whose meaning depends on the
/// type of request or response it is in.
/// The Contact header field has a role similar to the Location header field
/// in HTTP.
///
/// [[RFC3261, Section 20.10](https://datatracker.ietf.org/doc/html/rfc3261#section-20.10)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ContactHeader {
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
    crate::header::generic_header_accessors!(header);

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

impl std::fmt::Display for ContactHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for ContactHeader {
    fn eq(&self, other: &ContactHeader) -> bool {
        self.contacts == other.contacts
    }
}

/// Representation of the list of contacts of a `Contact` header.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub enum Contacts {
    /// Any contacts.
    Any,
    /// A list of contacts.
    Contacts(Vec<Contact>),
}

impl Contacts {
    /// Tell whether the contacts is the wildcard contact.
    pub fn is_any(&self) -> bool {
        matches!(self, Contacts::Any)
    }
}

impl From<Vec<Contact>> for Contacts {
    fn from(value: Vec<Contact>) -> Self {
        Self::Contacts(value)
    }
}

impl std::fmt::Display for Contacts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Any => "*".to_string(),
                Self::Contacts(contacts) => contacts
                    .iter()
                    .map(|contact| contact.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            }
        )
    }
}

impl PartialEq for Contacts {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, Self::Any) => true,
            (Self::Contacts(self_contacts), Self::Contacts(other_contacts)) => {
                let self_contacts: HashSet<_> = self_contacts.iter().collect();
                let other_contacts: HashSet<_> = other_contacts.iter().collect();
                self_contacts == other_contacts
            }
            _ => false,
        }
    }
}

impl IntoIterator for Contacts {
    type Item = Contact;
    type IntoIter = <Vec<Contact> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Any => vec![].into_iter(),
            Self::Contacts(contacts) => contacts.into_iter(),
        }
    }
}

impl Deref for Contacts {
    type Target = [Contact];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Any => &EMPTY_CONTACTS[..],
            Self::Contacts(contacts) => &contacts[..],
        }
    }
}

/// Representation of a contact in a `Contact` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct Contact {
    address: NameAddress,
    parameters: Vec<ContactParameter>,
}

impl Contact {
    pub(crate) fn new(address: NameAddress, parameters: Vec<ContactParameter>) -> Self {
        Contact {
            address,
            parameters,
        }
    }

    /// Get a reference to the address from the Contact.
    pub fn address(&self) -> &NameAddress {
        &self.address
    }

    /// Get a reference to the parameters from the Contact.
    pub fn parameters(&self) -> &Vec<ContactParameter> {
        &self.parameters
    }

    /// Get the value of the `q` parameter for the contact.
    pub fn q(&self) -> Option<f32> {
        self.parameters
            .iter()
            .find(|param| matches!(param, ContactParameter::Q(_)))
            .and_then(|param| param.q())
    }

    /// Get the value of the `expires` parameter for the contact.
    pub fn expires(&self) -> Option<u32> {
        self.parameters
            .iter()
            .find(|param| matches!(param, ContactParameter::Expires(_)))
            .and_then(|param| param.expires())
    }
}

impl std::fmt::Display for Contact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.address,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl PartialEq for Contact {
    fn eq(&self, other: &Self) -> bool {
        if self.address != other.address {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

impl Hash for Contact {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

/// Representation of a contact parameter.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialEqRefs)]
pub enum ContactParameter {
    /// A `q` parameter.
    Q(String),
    /// An `expires` parameter.
    Expires(String),
    /// Any other parameter.
    Other(GenericParameter),
}

impl ContactParameter {
    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Q(_) => "q",
            Self::Expires(_) => "expires",
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Q(value) => Some(value),
            Self::Expires(value) => Some(value),
            Self::Other(value) => value.value(),
        }
    }

    /// Get the q value of the parameter if this is a `q` parameter.
    pub fn q(&self) -> Option<f32> {
        match self {
            Self::Q(value) => value.parse().ok(),
            _ => None,
        }
    }

    /// Get the expires value of the parameter if this is an `expires`
    /// parameter.
    pub fn expires(&self) -> Option<u32> {
        match self {
            Self::Expires(value) => value.parse().ok(),
            _ => None,
        }
    }
}

impl std::fmt::Display for ContactParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.key(),
            if self.value().is_some() { "=" } else { "" },
            self.value().unwrap_or_default()
        )
    }
}

impl PartialOrd for ContactParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContactParameter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
    }
}

impl From<GenericParameter> for ContactParameter {
    fn from(value: GenericParameter) -> Self {
        Self::Other(value)
    }
}

#[cfg(test)]
mod tests {
    use super::ContactHeader;
    use crate::{
        header::{
            contact_header::Contacts,
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, Uri,
    };
    use claims::assert_ok;
    use std::str::FromStr;

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
                    Uri::from_str("sip:watson@worcester.bell-telephone.com").unwrap()
                );
                assert!(first_contact.address().uri().get_parameters().is_empty());
                assert_eq!(first_contact.parameters().len(), 2);
                assert!((first_contact.q().unwrap() - 0.7).abs() < 0.01);
                assert_eq!(first_contact.expires().unwrap(), 3600);
                let second_contact = contacts.next().unwrap();
                assert_eq!(second_contact.address().display_name(), Some("Mr. Watson"));
                assert_eq!(
                    second_contact.address().uri(),
                    Uri::from_str("mailto:watson@bell-telephone.com").unwrap()
                );
                assert!(second_contact.address().uri().get_parameters().is_empty());
                assert_eq!(second_contact.parameters().len(), 1);
                assert!((second_contact.q().unwrap() - 0.1).abs() < 0.01);
            },
        );
    }

    #[test]
    fn test_valid_contact_header_wildcard() {
        valid_header("Contact: *", |header| {
            assert_eq!(header.contacts(), Contacts::Any);
        });
    }

    #[test]
    fn test_valid_contact_header_compact_wildcard() {
        valid_header("m: *", |header| {
            assert_eq!(header.contacts(), Contacts::Any);
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
    fn test_contact_header_equality_same_wilcard_with_space_characters_differences() {
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
        let header = Header::from_str(
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
