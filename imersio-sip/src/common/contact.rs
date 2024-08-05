#![allow(missing_docs)]

use derive_more::IsVariant;
use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;
use std::ops::Deref;

use crate::utils::compare_vectors;
use crate::ContactParameter;
use crate::NameAddress;

static EMPTY_CONTACTS: Vec<Contact> = vec![];

/// Representation of the list of contacts of a `Contact` header.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
pub enum Contacts {
    /// Any contacts.
    Any,
    /// A list of contacts.
    Contacts(Vec<Contact>),
}

impl std::fmt::Display for Contacts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Any => "*".to_string(),
                Self::Contacts(contacts) => join(contacts, ", "),
            }
        )
    }
}

impl PartialEq for Contacts {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, Self::Any) => true,
            (Self::Contacts(self_contacts), Self::Contacts(other_contacts)) => {
                compare_vectors(self_contacts, other_contacts)
            }
            _ => false,
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

impl From<Vec<Contact>> for Contacts {
    fn from(value: Vec<Contact>) -> Self {
        Self::Contacts(value)
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
            join(&self.parameters, ";")
        )
    }
}

impl PartialEq for Contact {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && compare_vectors(self.parameters(), other.parameters())
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
