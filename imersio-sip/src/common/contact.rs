use itertools::join;
use std::hash::Hash;
use std::ops::Deref;

use crate::ContactParameter;
use crate::NameAddress;
use crate::utils::compare_vectors;

static EMPTY_CONTACTS: Vec<Contact> = vec![];

/// Representation of the list of contacts of a `Contact` header.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
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
#[derive(Clone, Debug, Eq)]
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

pub(crate) mod parser {
    use nom::{
        Parser,
        branch::alt,
        combinator::{map, opt, recognize},
        error::context,
        multi::many0,
        sequence::{delimited, pair, preceded},
    };

    use crate::{
        Contact, NameAddress, TokenString, Uri,
        common::{contact_parameter::parser::contact_params, wrapped_string::WrappedString},
        parser::{ParserResult, laquot, lws, quoted_string, raquot, semi, token},
        uris::{absolute_uri::parser::absolute_uri, sip_uri::parser::sip_uri},
    };

    pub(crate) fn addr_spec(input: &str) -> ParserResult<&str, Uri> {
        context(
            "addr_spec",
            alt((map(sip_uri, Uri::Sip), map(absolute_uri, Uri::Absolute))),
        )
        .parse(input)
    }

    fn display_name(input: &str) -> ParserResult<&str, WrappedString<TokenString>> {
        context(
            "display_name",
            alt((
                quoted_string,
                map(recognize(many0(pair(token, lws))), |v| {
                    WrappedString::new_not_wrapped(TokenString::new(v.trim_end()))
                }),
            )),
        )
        .parse(input)
    }

    pub(crate) fn name_addr(input: &str) -> ParserResult<&str, NameAddress> {
        context(
            "name_addr",
            map(
                pair(opt(display_name), delimited(laquot, addr_spec, raquot)),
                |(display_name, uri)| NameAddress::new(uri, display_name),
            ),
        )
        .parse(input)
    }

    pub(crate) fn contact_param(input: &str) -> ParserResult<&str, Contact> {
        context(
            "contact_param",
            map(
                pair(
                    alt((name_addr, map(addr_spec, |uri| NameAddress::new(uri, None)))),
                    many0(preceded(semi, contact_params)),
                ),
                |(address, params)| Contact::new(address, params),
            ),
        )
        .parse(input)
    }
}
