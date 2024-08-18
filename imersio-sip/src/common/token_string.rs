use std::convert::TryFrom;

use derive_more::{Deref, Display};
use nom::error::convert_error;

use crate::parser::token;
use crate::SipError;

/// Representation of a URI scheme value accepting only the valid characters.
#[derive(Clone, Debug, Deref, Display, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TokenString(String);

impl TokenString {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for TokenString {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<&str> for TokenString {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match token(value) {
            Ok((rest, token_string)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(token_string)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidTokenString(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidTokenString(format!(
                "Incomplete token string `{}`",
                value
            ))),
        }
    }
}
