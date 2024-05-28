use std::hash::Hash;

use crate::{utils::partial_eq_refs, Uri};

#[derive(Clone, Debug, Eq)]
pub struct NameAddress {
    display_name: Option<String>,
    uri: Uri,
}

impl NameAddress {
    pub(crate) fn new(uri: Uri, display_name: Option<String>) -> Self {
        Self { display_name, uri }
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

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
                Some(display_name) => format!("\"{display_name}\" "),
                None => "".to_string(),
            },
            self.uri
        )
    }
}

impl PartialEq for NameAddress {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

partial_eq_refs!(NameAddress);

impl Hash for NameAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
    }
}

impl From<NameAddress> for Uri {
    fn from(value: NameAddress) -> Self {
        value.uri
    }
}
