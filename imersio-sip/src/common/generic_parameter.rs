use std::hash::Hash;

use partial_eq_refs::PartialEqRefs;

/// Representation of a generic parameter.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct GenericParameter {
    key: String,
    value: Option<String>,
}

impl GenericParameter {
    /// Create a `GenericParameter`.
    pub fn new<S: Into<String>>(key: S, value: Option<S>) -> Self {
        Self {
            key: key.into(),
            value: value.map(Into::into),
        }
    }

    /// Get the key of the generic parameter.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the value of the generic parameter.
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

impl std::fmt::Display for GenericParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.key,
            if self.value.is_some() { "=" } else { "" },
            self.value.as_deref().unwrap_or_default()
        )
    }
}

impl PartialEq for GenericParameter {
    fn eq(&self, other: &GenericParameter) -> bool {
        self.key().eq_ignore_ascii_case(other.key())
            && self.value().map(|v| v.to_ascii_lowercase())
                == other.value().map(|v| v.to_ascii_lowercase())
    }
}

impl Hash for GenericParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        self.value().map(|v| v.to_ascii_lowercase()).hash(state);
    }
}
