//! Parsing and generation of the userinfo part of a SIP URI.  

use crate::uris::user_info::parser::{is_password_special_char, is_user_unreserved};
use crate::{parser::is_unreserved, utils::escape};

/// Representation of an userinfo of a SIP URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

pub(crate) mod parser {
    use crate::parser::{escaped, take1, unreserved, ParserResult};
    use crate::UserInfo;
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

    fn user_unreserved(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| is_user_unreserved(*c))(input)
    }

    fn user(input: &str) -> ParserResult<&str, String> {
        context(
            "user",
            map(many1(alt((unreserved, escaped, user_unreserved))), |user| {
                user.iter().collect::<String>()
            }),
        )(input)
    }

    #[inline]
    pub(crate) fn is_password_special_char(c: char) -> bool {
        "&=+$,".contains(c)
    }

    fn password_special_char(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| is_password_special_char(*c))(input)
    }

    fn password(input: &str) -> ParserResult<&str, String> {
        context(
            "password",
            map(
                many0(alt((unreserved, escaped, password_special_char))),
                |password| password.iter().collect::<String>(),
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
