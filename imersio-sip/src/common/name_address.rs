use derive_partial_eq_extras::PartialEqExtras;
use std::hash::Hash;

use super::wrapped_string::WrappedString;
use crate::{TokenString, Uri};

/// Representation of name address, that is the conjunction of a display name and a uri.
#[derive(Clone, Debug, Eq, PartialEqExtras)]
pub struct NameAddress {
    #[partial_eq_ignore]
    display_name: Option<WrappedString<TokenString>>,
    uri: Uri,
}

impl NameAddress {
    pub(crate) fn new(uri: Uri, display_name: Option<WrappedString<TokenString>>) -> Self {
        let display_name = display_name.filter(|display_name| !display_name.is_empty());
        Self { display_name, uri }
    }

    /// Get the display name of the name address.
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    /// Get the uri of the name address.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }
}

impl std::fmt::Display for NameAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}<{}>",
            match &self.display_name {
                Some(display_name) => format!("{} ", display_name),
                None => "".to_string(),
            },
            self.uri
        )
    }
}

impl Hash for NameAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
    }
}

impl From<Uri> for NameAddress {
    fn from(value: Uri) -> Self {
        Self {
            display_name: None,
            uri: value,
        }
    }
}

impl From<NameAddress> for Uri {
    fn from(value: NameAddress) -> Self {
        value.uri
    }
}
