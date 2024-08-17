use derive_more::{Deref, Display};
use derive_partial_eq_extras::PartialEqExtras;
use std::hash::{Hash, Hasher};

/// Representation of a stale parameter.
#[derive(Debug, Clone, Deref, Display, Eq, PartialEqExtras)]
#[display("{}", str_value)]
pub struct Stale {
    #[partial_eq_ignore]
    str_value: String,
    #[deref]
    value: bool,
}

impl Stale {
    pub(crate) fn new<S: Into<String>>(str_value: S, value: bool) -> Self {
        Self {
            str_value: str_value.into(),
            value,
        }
    }
}

impl Hash for Stale {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl From<bool> for Stale {
    fn from(value: bool) -> Self {
        Self {
            str_value: if value { "true".into() } else { "false".into() },
            value,
        }
    }
}
