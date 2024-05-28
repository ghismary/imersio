use std::hash::Hash;

use crate::utils::partial_eq_refs;

#[derive(Clone, Debug, Eq)]
pub enum MessageQop {
    Auth,
    AuthInt,
    Other(String),
}

impl MessageQop {
    pub(crate) fn new<S: Into<String>>(qop: S) -> Self {
        let qop: String = qop.into();
        match qop.to_ascii_lowercase().as_str() {
            "auth" => Self::Auth,
            "auth-int" => Self::AuthInt,
            _ => Self::Other(qop),
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Auth => "auth",
            Self::AuthInt => "auth-int",
            Self::Other(value) => value,
        }
    }
}

impl std::fmt::Display for MessageQop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq<MessageQop> for MessageQop {
    fn eq(&self, other: &MessageQop) -> bool {
        match (self, other) {
            (Self::Auth, Self::Auth) | (Self::AuthInt, Self::AuthInt) => true,
            (Self::Other(a), Self::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

partial_eq_refs!(MessageQop);

impl Hash for MessageQop {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}
