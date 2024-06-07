use crate::common::header_value_collection::HeaderValueCollection;
use partial_eq_refs::PartialEqRefs;

use super::{generic_header::GenericHeader, HeaderAccessor};

/// Representation of an In-Reply-To header.
///
/// The In-Reply-To header field enumerates the Call-IDs that this call references or returns.
/// These Call-IDs may have been cached by the client then included in this header field in a
/// return call.
///
/// [[RFC3261, Section 20.21](https://datatracker.ietf.org/doc/html/rfc3261#section-20.21)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct InReplyToHeader {
    header: GenericHeader,
    call_ids: CallIds,
}

impl InReplyToHeader {
    pub(crate) fn new<S: Into<String>>(header: GenericHeader, call_ids: Vec<S>) -> Self {
        Self {
            header,
            call_ids: call_ids
                .into_iter()
                .map(|id| id.into())
                .collect::<Vec<String>>()
                .into(),
        }
    }

    /// Get the list of call IDs from the In-Reply-To header.
    pub fn call_ids(&self) -> &CallIds {
        &self.call_ids
    }
}

impl HeaderAccessor for InReplyToHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("In-Reply-To")
    }
    fn normalized_value(&self) -> String {
        self.call_ids.to_string()
    }
}

impl std::fmt::Display for InReplyToHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for InReplyToHeader {
    fn eq(&self, other: &InReplyToHeader) -> bool {
        self.call_ids == other.call_ids
    }
}

/// Representation of the list of call IDs in a `In-Reply-To` header.
///
/// This is usable as an iterator.
pub type CallIds = HeaderValueCollection<String>;

#[cfg(test)]
mod tests {
    use super::InReplyToHeader;
    use crate::{
        header::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header,
    };
    use claims::assert_ok;
    use std::str::FromStr;

    valid_header!(InReplyTo, InReplyToHeader, "In-Reply-To");
    header_equality!(InReplyTo, "In-Reply-To");
    header_inequality!(InReplyTo, "In-Reply-To");

    #[test]
    fn test_valid_in_reply_to_header_with_a_single_call_id() {
        valid_header("In-Reply-To: 70710@saturn.bell-tel.com", |header| {
            assert_eq!(header.call_ids().len(), 1);
            assert_eq!(
                header.call_ids().first().unwrap(),
                &"70710@saturn.bell-tel.com".to_string()
            );
        });
    }

    #[test]
    fn test_valid_in_reply_to_header_with_several_call_ids() {
        valid_header(
            "In-Reply-To: 70710@saturn.bell-tel.com, 17320@saturn.bell-tel.com",
            |header| {
                assert_eq!(header.call_ids().len(), 2);
                assert_eq!(
                    header.call_ids().first().unwrap(),
                    &"70710@saturn.bell-tel.com".to_string()
                );
                assert_eq!(
                    header.call_ids().last().unwrap(),
                    &"17320@saturn.bell-tel.com".to_string()
                );
            },
        );
    }

    #[test]
    fn test_invalid_in_reply_to_header_empty() {
        invalid_header("In-Reply-To:");
    }

    #[test]
    fn test_invalid_in_reply_to_header_empty_with_space_characters() {
        invalid_header("In-Reply-To:    ");
    }

    #[test]
    fn test_invalid_in_reply_to_header_with_invalid_character() {
        invalid_header("In-Reply-To: 😁");
    }

    #[test]
    fn test_in_reply_to_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            "In-Reply-To: 70710@saturn.bell-tel.com, 17320@saturn.bell-tel.com",
            "In-Reply-To  :    70710@saturn.bell-tel.com        , 17320@saturn.bell-tel.com",
        );
    }

    #[test]
    fn test_in_reply_to_header_equality_same_header_with_call_ids_in_a_different_order() {
        header_equality(
            "In-Reply-To: 70710@saturn.bell-tel.com, 17320@saturn.bell-tel.com",
            "In-Reply-To: 17320@saturn.bell-tel.com, 70710@saturn.bell-tel.com",
        );
    }

    #[test]
    fn test_in_reply_to_header_inequality_different_values() {
        header_inequality(
            "In-Reply-To: 70710@saturn.bell-tel.com",
            "In-Reply-To: 17320@saturn.bell-tel.com",
        );
    }

    #[test]
    fn test_in_reply_to_header_inequality_with_first_header_having_more_call_ids_than_the_second() {
        header_inequality(
            "In-Reply-To: 70710@saturn.bell-tel.com, 17320@saturn.bell-tel.com",
            "In-Reply-To: 70710@saturn.bell-tel.com",
        );
    }

    #[test]
    fn test_in_reply_to_header_inequality_with_first_header_having_less_call_ids_than_the_second() {
        header_inequality(
            "In-Reply-To: 70710@saturn.bell-tel.com",
            "In-Reply-To: 70710@saturn.bell-tel.com, 17320@saturn.bell-tel.com",
        );
    }

    #[test]
    fn test_in_reply_to_header_to_string() {
        let header = Header::from_str(
            "in-reply-to  :   70710@saturn.bell-tel.com   , 17320@saturn.bell-tel.com",
        );
        if let Header::InReplyTo(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "in-reply-to  :   70710@saturn.bell-tel.com   , 17320@saturn.bell-tel.com"
            );
            assert_eq!(
                header.to_normalized_string(),
                "In-Reply-To: 70710@saturn.bell-tel.com, 17320@saturn.bell-tel.com"
            );
            assert_eq!(
                header.to_compact_string(),
                "In-Reply-To: 70710@saturn.bell-tel.com, 17320@saturn.bell-tel.com"
            );
        }
    }
}
