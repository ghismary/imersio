use derive_more::Display;
use nom::error::convert_error;
use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::Error;

/// Representation of the list of call IDs in a `In-Reply-To` header.
///
/// This is usable as an iterator.
pub type CallIds = ValueCollection<CallId>;

/// Representation of a call id contained in a `Call-Id` or `In-Reply-To` header.
#[derive(Clone, Debug, Display, Eq, PartialEqRefs)]
#[display(fmt = "{}", "self.0.to_ascii_lowercase()")]
pub struct CallId(String);

impl CallId {
    pub(crate) fn new<S: Into<String>>(callid: S) -> Self {
        Self(callid.into())
    }
}

impl PartialEq for CallId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<str> for CallId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<CallId> for str {
    fn eq(&self, other: &CallId) -> bool {
        self == other.0
    }
}

impl PartialEq<&str> for CallId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<CallId> for &str {
    fn eq(&self, other: &CallId) -> bool {
        *self == other.0
    }
}

impl PartialOrd for CallId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CallId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for CallId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl AsRef<str> for CallId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for CallId {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::callid(value) {
            Ok((rest, call_id)) => {
                if !rest.is_empty() {
                    Err(Error::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(call_id)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(Error::InvalidCallId(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidCallId(format!(
                "Incomplete call id `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use crate::parser::{word, ParserResult};
    use crate::CallId;
    use nom::{
        bytes::complete::tag,
        combinator::{map, opt, recognize},
        error::context,
        sequence::pair,
    };

    pub(crate) fn callid(input: &str) -> ParserResult<&str, CallId> {
        context(
            "callid",
            map(
                recognize(pair(word, opt(pair(tag("@"), word)))),
                CallId::new,
            ),
        )(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_call_id_eq() {
        assert_eq!(
            CallId::try_from("f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com").unwrap(),
            "f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com"
        );
    }

    #[test]
    fn test_call_id_not_eq_different_case() {
        assert_ne!(
            CallId::try_from("f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com").unwrap(),
            "F81D4FAE-7DEC-11D0-A765-00A0C91E6BF6@foo.bar.com"
        );
    }

    #[test]
    fn test_valid_call_id() {
        assert_ok!(CallId::try_from(
            "f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com"
        ));
    }

    #[test]
    fn test_valid_call_id_without_at() {
        assert_ok!(CallId::try_from("f81d4fae-7dec-11d0-a765-00a0c91e6bf6"));
    }

    #[test]
    fn test_invalid_call_id_empty() {
        assert_err!(CallId::try_from(""));
    }

    #[test]
    fn test_invalid_call_id_with_invalid_character() {
        assert_err!(CallId::try_from("😁"));
    }

    #[test]
    fn test_invalid_call_id_with_at_but_no_second_word() {
        assert_err!(CallId::try_from("f81d4fae-7dec-11d0-a765-00a0c91e6bf6@"));
    }

    #[test]
    fn test_valid_call_id_with_remaining_data() {
        assert!(
            CallId::try_from("f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com anything")
                .is_err_and(|e| e == Error::RemainingUnparsedData(" anything".to_string()))
        );
    }
}
