use std::cmp::Ordering;
use std::hash::Hash;

use crate::{SipError, TokenString};

/// Representation of a user type included in a `user` uri parameter.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum UserType {
    /// Phone user.
    Phone,
    /// IP user.
    Ip,
    /// Any other user type.
    Other(TokenString),
}

impl UserType {
    pub(crate) fn new(user_type: TokenString) -> Self {
        match user_type.to_ascii_lowercase().as_str() {
            "phone" => Self::Phone,
            "ip" => Self::Ip,
            _ => Self::Other(user_type),
        }
    }

    /// Get the value of the user type.
    pub fn value(&self) -> &str {
        match self {
            Self::Phone => "phone",
            Self::Ip => "ip",
            Self::Other(value) => value,
        }
    }
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq for UserType {
    fn eq(&self, other: &UserType) -> bool {
        match (self, other) {
            (Self::Phone, Self::Phone) | (Self::Ip, Self::Ip) => true,
            (Self::Other(a), Self::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialOrd for UserType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(other.value())
    }
}

impl Hash for UserType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().to_ascii_lowercase().hash(state);
    }
}

impl TryFrom<&str> for UserType {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(UserType::new(TokenString::try_from(value)?))
    }
}
