use derive_more::IsVariant;
use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;
use std::hash::Hash;

/// Representation of the `handling` parameter of a `DispositionType`.
#[derive(Clone, Debug, Eq, IsVariant, PartialEqRefs)]
pub enum Handling {
    /// The handling of the content type is optional.
    Optional,
    /// The handling of the content type is required.
    Required,
    /// Any extension value.
    Other(String),
}

impl Handling {
    pub(crate) fn new<S: Into<String>>(handling: S) -> Handling {
        let handling: String = handling.into();
        match handling.to_ascii_lowercase().as_str() {
            "optional" => Self::Optional,
            "required" => Self::Required,
            _ => Self::Other(handling),
        }
    }

    /// Get the value of the `HandlingValue.`
    pub fn value(&self) -> &str {
        match self {
            Self::Optional => "optional",
            Self::Required => "required",
            Self::Other(value) => value,
        }
    }
}

impl std::fmt::Display for Handling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq for Handling {
    fn eq(&self, other: &Handling) -> bool {
        match (self, other) {
            (Self::Optional, Self::Optional) | (Self::Required, Self::Required) => true,
            (Self::Other(svalue), Self::Other(ovalue)) => svalue.eq_ignore_ascii_case(ovalue),
            _ => false,
        }
    }
}

impl PartialOrd for Handling {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Handling {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(other.value())
    }
}

impl Hash for Handling {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}
