use std::cmp::Ordering;

use crate::utils::partial_eq_refs;

use super::HeaderAccessor;

#[derive(Clone, Debug, Eq)]
pub struct GenericHeader {
    name: String,
    separator: String,
    value: String,
}

impl GenericHeader {
    pub(crate) fn new<S: Into<String>>(name: S, separator: S, value: S) -> Self {
        Self {
            name: name.into(),
            separator: separator.into(),
            value: value.into(),
        }
    }
}

impl HeaderAccessor for GenericHeader {
    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn separator(&self) -> &str {
        self.separator.as_str()
    }
    fn value(&self) -> &str {
        self.value.as_str()
    }

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        None
    }
    fn normalized_value(&self) -> String {
        self.value.clone()
    }
}

impl std::fmt::Display for GenericHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.name, self.separator, self.value)
    }
}

impl PartialEq for GenericHeader {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq_ignore_ascii_case(&other.name) && self.value.eq_ignore_ascii_case(&other.value)
    }
}

partial_eq_refs!(GenericHeader);

impl PartialOrd for GenericHeader {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GenericHeader {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self
            .name
            .to_ascii_lowercase()
            .cmp(&other.name.to_ascii_lowercase())
        {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value
            .to_ascii_lowercase()
            .cmp(&other.value.to_ascii_lowercase())
    }
}
