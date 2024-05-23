use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::{common::NameAddress, GenericParameter};

#[derive(Clone, Debug)]
pub enum ContactHeader {
    Any,
    Contacts(Vec<Contact>),
}

impl ContactHeader {
    pub fn contacts(&self) -> &Vec<Contact> {
        static EMPTY_CONTACTS: Vec<Contact> = vec![];
        match self {
            Self::Any => &EMPTY_CONTACTS,
            Self::Contacts(contacts) => contacts,
        }
    }
}

impl std::fmt::Display for ContactHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Contact: {}",
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

impl PartialEq for ContactHeader {
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

impl PartialEq<&ContactHeader> for ContactHeader {
    fn eq(&self, other: &&ContactHeader) -> bool {
        self == *other
    }
}

impl PartialEq<ContactHeader> for &ContactHeader {
    fn eq(&self, other: &ContactHeader) -> bool {
        *self == other
    }
}

impl Eq for ContactHeader {}

#[derive(Clone, Debug)]
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

    pub fn q(&self) -> Option<f32> {
        self.parameters
            .iter()
            .find(|param| matches!(param, ContactParameter::Q(_)))
            .and_then(|param| param.q())
    }

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

impl PartialEq<&Contact> for Contact {
    fn eq(&self, other: &&Contact) -> bool {
        self == *other
    }
}

impl PartialEq<Contact> for &Contact {
    fn eq(&self, other: &Contact) -> bool {
        *self == other
    }
}

impl Eq for Contact {}

impl Hash for Contact {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ContactParameter {
    Q(String),
    Expires(String),
    Other(String, Option<String>),
}

impl ContactParameter {
    pub fn key(&self) -> &str {
        match self {
            Self::Q(_) => "q",
            Self::Expires(_) => "expires",
            Self::Other(key, _) => key,
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Q(value) => Some(value),
            Self::Expires(value) => Some(value),
            Self::Other(_, value) => value.as_deref(),
        }
    }

    pub fn q(&self) -> Option<f32> {
        match self {
            Self::Q(value) => value.parse().ok(),
            _ => None,
        }
    }

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

impl PartialEq<&ContactParameter> for ContactParameter {
    fn eq(&self, other: &&ContactParameter) -> bool {
        self == *other
    }
}

impl PartialEq<ContactParameter> for &ContactParameter {
    fn eq(&self, other: &ContactParameter) -> bool {
        *self == other
    }
}

impl PartialOrd for ContactParameter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
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
        Self::Other(value.key().to_string(), value.value().map(Into::into))
    }
}

#[cfg(test)]
mod tests {
    use crate::{header::contact_header::ContactHeader, Header, Uri};
    use std::str::FromStr;

    #[test]
    fn test_valid_contact_header() {
        // Valid Contact header.
        let header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600, "Mr. Watson" <mailto:watson@bell-telephone.com> ;q=0.1"#,
        );
        assert!(header.is_ok());
        if let Header::Contact(header) = header.unwrap() {
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
        } else {
            panic!("Not an Contact header");
        }

        // Valid wildcard Contact header.
        let header = Header::from_str("Contact: *");
        assert!(header.is_ok());
        if let Header::Contact(header) = header.unwrap() {
            assert_eq!(header, ContactHeader::Any);
        } else {
            panic!("Not an Contact header");
        }

        // Valid abbreviated wildcard Contact header.
        let header = Header::from_str("m: *");
        assert!(header.is_ok());
        if let Header::Contact(header) = header.unwrap() {
            assert_eq!(header, ContactHeader::Any);
        } else {
            panic!("Not an Contact header");
        }
    }

    #[test]
    fn test_invalid_contact_header() {
        // Empty Contact header.
        let header = Header::from_str("Contact:");
        assert!(header.is_err());

        // Empty Contact header with spaces.
        let header = Header::from_str("Contact:    ");
        assert!(header.is_err());

        // Contact header with invalid character.
        let header = Header::from_str("Contact: 😁");
        assert!(header.is_err());
    }

    #[test]
    fn test_contact_header_equality() {
        // Same contact headers, just some minor spaces differences.
        let first_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
        let second_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com> ;q=0.7;expires=3600"#,
        );
        if let (Header::Contact(first_header), Header::Contact(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Contact header");
        }

        // Contact headers with contacts in a different order.
        let first_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600, "Mr. Watson" <mailto:watson@bell-telephone.com> ;q=0.1"#,
        );
        let second_header = Header::from_str(
            r#"Contact: "Mr. Watson" <mailto:watson@bell-telephone.com> ;q=0.1, "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
        if let (Header::Contact(first_header), Header::Contact(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Contact header");
        }

        // Same wildcard contact headers.
        let first_header = Header::from_str("Contact: *");
        let second_header = Header::from_str("Contact: *");
        if let (Header::Contact(first_header), Header::Contact(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Contact header");
        }

        // Same contact headers with different display names.
        let first_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
        let second_header = Header::from_str(
            r#"Contact: "Mrs. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
        if let (Header::Contact(first_header), Header::Contact(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Contact header");
        }
    }

    #[test]
    fn test_contact_header_inequality() {
        // Different contact uris.
        let first_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
        let second_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@manchester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
        if let (Header::Contact(first_header), Header::Contact(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Contact header");
        }

        // Different q parameters.
        let first_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
        let second_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.5; expires=3600"#,
        );
        if let (Header::Contact(first_header), Header::Contact(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Contact header");
        }

        // Different expires parameters.
        let first_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3600"#,
        );
        let second_header = Header::from_str(
            r#"Contact: "Mr. Watson" <sip:watson@worcester.bell-telephone.com>;q=0.7; expires=3200"#,
        );
        if let (Header::Contact(first_header), Header::Contact(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Contact header");
        }
    }
}
