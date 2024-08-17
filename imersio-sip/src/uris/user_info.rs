//! Parsing and generation of the userinfo part of a SIP URI.  

use derive_more::{Deref, Display};

use crate::uris::user_info::parser::{is_password_special_char, is_user_unreserved};
use crate::{
    parser::{is_unreserved, ESCAPED_CHARS},
    utils::escape,
    SipError,
};

/// Representation of a URI user value accepting only the valid characters.
#[derive(Clone, Debug, Deref, Display, Eq, Hash, PartialEq)]
pub struct UserString(String);

impl UserString {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl TryFrom<&str> for UserString {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Do not use the parser because of the escaped characters, instead check that each
        // character of the given value can be escaped.
        if value.chars().all(|c| {
            let idx: Result<u8, _> = c.try_into();
            match idx {
                Ok(idx) => ESCAPED_CHARS[idx as usize] != '\0',
                Err(_) => false,
            }
        }) {
            Ok(Self::new(value))
        } else {
            Err(SipError::InvalidUriUser(value.to_string()))
        }
    }
}

/// Representation of a URI password value accepting only the valid characters.
#[derive(Clone, Debug, Deref, Display, Eq, Hash, PartialEq)]
pub struct PasswordString(String);

impl PasswordString {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl TryFrom<&str> for PasswordString {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Do not use the parser because of the escaped characters, instead check that each
        // character of the given value can be escaped.
        if value.chars().all(|c| {
            let idx: Result<u8, _> = c.try_into();
            match idx {
                Ok(idx) => ESCAPED_CHARS[idx as usize] != '\0',
                Err(_) => false,
            }
        }) {
            Ok(Self::new(value))
        } else {
            Err(SipError::InvalidUriPassword(value.to_string()))
        }
    }
}

/// Representation of an userinfo of a SIP URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UserInfo {
    user: UserString,
    password: Option<PasswordString>,
}

impl UserInfo {
    pub(crate) fn new(user: UserString, password: Option<PasswordString>) -> Self {
        Self { user, password }
    }

    /// Get the user part of the user info as a string slice.
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Get the password part of the user info as a string slice.
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref().map(|p| p.as_str())
    }
}

impl std::fmt::Display for UserInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            escape(self.user(), |b| {
                is_unreserved(b) || is_user_unreserved(b)
            }),
            if self.password().is_some() { ":" } else { "" },
            escape(self.password().unwrap_or_default(), |b| {
                is_unreserved(b) || is_password_special_char(b)
            })
        )
    }
}

pub(crate) mod parser {
    use crate::parser::{escaped, take1, unreserved, ParserResult};
    use crate::{PasswordString, UserInfo, UserString};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, opt, verify},
        error::context,
        multi::{many0, many1},
        sequence::{preceded, tuple},
    };

    #[inline]
    pub(crate) fn is_user_unreserved(c: char) -> bool {
        "&=+$,;?/".contains(c)
    }

    #[inline]
    fn user_unreserved(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| is_user_unreserved(*c))(input)
    }

    pub(super) fn user(input: &str) -> ParserResult<&str, UserString> {
        context(
            "user",
            map(many1(alt((unreserved, escaped, user_unreserved))), |user| {
                UserString::new(user.iter().collect::<String>())
            }),
        )(input)
    }

    #[inline]
    pub(crate) fn is_password_special_char(c: char) -> bool {
        "&=+$,".contains(c)
    }

    #[inline]
    fn password_special_char(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| is_password_special_char(*c))(input)
    }

    pub(super) fn password(input: &str) -> ParserResult<&str, PasswordString> {
        context(
            "password",
            map(
                many0(alt((unreserved, escaped, password_special_char))),
                |password| PasswordString::new(password.iter().collect::<String>()),
            ),
        )(input)
    }

    pub(crate) fn userinfo(input: &str) -> ParserResult<&str, UserInfo> {
        context(
            "userinfo",
            map(
                tuple((
                    user, // TODO: alt((user, telephone_subscriber)),
                    opt(preceded(tag(":"), password)),
                    tag("@"),
                )),
                |(user, password, _)| UserInfo::new(user, password),
            ),
        )(input)
    }
}
#[cfg(test)]
mod tests {
    use crate::{PasswordString, UserInfo, UserString};
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_valid_user_string() {
        let user_string = UserString::try_from("bob");
        assert_ok!(&user_string);
        if let Ok(user_string) = user_string {
            assert_eq!(user_string.as_str(), "bob");
            assert_eq!(format!("{}", user_string), "bob");
        }
    }

    #[test]
    fn test_valid_user_string_needing_escaping() {
        let user_string = UserString::try_from("a username");
        assert_ok!(&user_string);
        if let Ok(user_string) = user_string {
            assert_eq!(user_string.as_str(), "a username");
            assert_eq!(format!("{}", user_string), "a username");
        }
    }

    #[test]
    fn test_invalid_user_string() {
        assert_err!(UserString::try_from("Éric"));
    }

    #[test]
    fn test_valid_password_string() {
        let password_string = PasswordString::try_from("password");
        assert_ok!(&password_string);
        if let Ok(password_string) = password_string {
            assert_eq!(password_string.as_str(), "password");
            assert_eq!(format!("{}", password_string), "password");
        }
    }

    #[test]
    fn test_valid_password_string_needing_escaping() {
        let password_string = PasswordString::try_from("secret# word$");
        assert_ok!(&password_string);
        if let Ok(password_string) = password_string {
            assert_eq!(password_string.as_str(), "secret# word$");
            assert_eq!(format!("{}", password_string), "secret# word$");
        }
    }

    #[test]
    fn test_invalid_password_string() {
        assert_err!(PasswordString::try_from("mot crypté"));
    }

    #[test]
    fn test_userinfo_display() {
        let user_string = UserString::try_from("bob").unwrap();
        let password_string = PasswordString::try_from("password").unwrap();
        let user_info = UserInfo::new(user_string, Some(password_string));
        assert_eq!(format!("{}", user_info), "bob:password");
    }

    #[test]
    fn test_userinfo_display_without_password() {
        let user_string = UserString::try_from("bob").unwrap();
        let user_info = UserInfo::new(user_string, None);
        assert_eq!(format!("{}", user_info), "bob");
    }

    #[test]
    fn test_userinfo_display_with_escaped_chars() {
        let user_string = UserString::try_from("a username").unwrap();
        let password_string = PasswordString::try_from("secret# word$").unwrap();
        let user_info = UserInfo::new(user_string, Some(password_string));
        assert_eq!(format!("{}", user_info), "a%20username:secret%23%20word$");
    }
}
