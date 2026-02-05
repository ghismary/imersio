use crate::TokenString;
use std::cmp::Ordering;

/// Representation of a disposition type from a `Content-Disposition` header.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum DispositionType {
    /// The value `render` indicates that the body part should be displayed or
    /// otherwise rendered to the user.
    Render,
    /// The value `session` indicates that the body part describes a session,
    /// for either calls or early (pre-call) media.
    Session,
    /// The value `icon` indicates that the body part contains an image
    /// suitable as an iconic representation of the caller or callee that
    /// could be rendered informationally by a user agent when a message has
    /// been received, or persistently while a dialog takes place.
    Icon,
    /// The value `alert`` indicates that the body part contains information,
    /// such as an audio clip, that should be rendered by the user agent in an
    /// attempt to alert the user to the receipt of a request.
    Alert,
    /// Any other extension disposition type.
    Other(TokenString),
}

impl DispositionType {
    pub(crate) fn new(r#type: TokenString) -> DispositionType {
        match r#type.to_ascii_lowercase().as_ref() {
            "render" => Self::Render,
            "session" => Self::Session,
            "icon" => Self::Icon,
            "alert" => Self::Alert,
            _ => Self::Other(r#type),
        }
    }
}

impl std::fmt::Display for DispositionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Render => "render",
                Self::Session => "session",
                Self::Icon => "icon",
                Self::Alert => "alert",
                Self::Other(value) => value,
            }
        )
    }
}

impl PartialEq for DispositionType {
    fn eq(&self, other: &DispositionType) -> bool {
        match (self, other) {
            (Self::Render, Self::Render)
            | (Self::Session, Self::Session)
            | (Self::Icon, Self::Icon)
            | (Self::Alert, Self::Alert) => true,
            (Self::Other(self_value), Self::Other(other_value)) => {
                self_value.eq_ignore_ascii_case(other_value)
            }
            _ => false,
        }
    }
}

impl PartialOrd for DispositionType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DispositionType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

pub(crate) mod parser {
    use nom::{Parser, branch::alt, bytes::complete::tag_no_case, combinator::map};

    use crate::{
        DispositionType, TokenString,
        parser::{ParserResult, token},
    };

    #[inline]
    fn disp_extension_token(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }

    pub(crate) fn disp_type(input: &str) -> ParserResult<&str, DispositionType> {
        map(
            alt((
                map(tag_no_case("render"), TokenString::new),
                map(tag_no_case("session"), TokenString::new),
                map(tag_no_case("icon"), TokenString::new),
                map(tag_no_case("alert"), TokenString::new),
                disp_extension_token,
            )),
            DispositionType::new,
        )
        .parse(input)
    }
}
