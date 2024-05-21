//! TODO

/// Representation of a generic parameter.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenericParameter {
    key: String,
    value: Option<String>,
}

impl GenericParameter {
    /// Create a `GenericParam`.
    pub fn new(key: String, value: Option<String>) -> Self {
        Self { key, value }
    }

    /// Get the key of the `GenericParam`.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the value of the `GenericParam`.
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

impl PartialEq<&GenericParameter> for GenericParameter {
    fn eq(&self, other: &&GenericParameter) -> bool {
        self == *other
    }
}

impl PartialEq<GenericParameter> for &GenericParameter {
    fn eq(&self, other: &GenericParameter) -> bool {
        *self == other
    }
}
