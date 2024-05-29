use partial_eq_refs::PartialEqRefs;

use crate::{
    parser::is_unreserved,
    uri::parser::{is_password_special_char, is_user_unreserved},
    utils::escape,
};

/// Representation of an userinfo of a SIP URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialEqRefs)]
pub struct UserInfo {
    user: String,
    password: Option<String>,
}

impl UserInfo {
    pub(crate) fn new<S: Into<String>>(user: S, password: Option<S>) -> Self {
        Self {
            user: user.into(),
            password: password.map(Into::into),
        }
    }

    /// Get the user part of the `UserInfo`.
    pub fn get_user(&self) -> &str {
        &self.user
    }

    /// Get the password part of the `UserInfo`.
    pub fn get_password(&self) -> Option<&str> {
        self.password.as_deref()
    }
}

impl std::fmt::Display for UserInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            escape(&self.user, |b| {
                is_unreserved(b) || is_user_unreserved(b)
            }),
            if self.password.is_some() { ":" } else { "" },
            escape(self.password.as_deref().unwrap_or_default(), |b| {
                is_unreserved(b) || is_password_special_char(b)
            })
        )
    }
}
