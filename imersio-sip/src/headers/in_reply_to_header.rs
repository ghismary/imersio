//! SIP In-Reply-To header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{CallId, CallIds};

/// Representation of an In-Reply-To header.
///
/// The In-Reply-To header field enumerates the Call-IDs that this call references or returns.
/// These Call-IDs may have been cached by the client then included in this header field in a
/// return call.
///
/// [[RFC3261, Section 20.21](https://datatracker.ietf.org/doc/html/rfc3261#section-20.21)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct InReplyToHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    call_ids: CallIds,
}

impl InReplyToHeader {
    pub(crate) fn new(header: GenericHeader, call_ids: Vec<CallId>) -> Self {
        Self {
            header,
            call_ids: call_ids.into(),
        }
    }

    /// Get the list of call IDs from the In-Reply-To header.
    pub fn call_ids(&self) -> &CallIds {
        &self.call_ids
    }
}

impl HeaderAccessor for InReplyToHeader {
    crate::headers::generic_header_accessors!(header);

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

pub(crate) mod parser {
    use crate::common::call_id::parser::callid;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{Header, InReplyToHeader};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn in_reply_to(input: &str) -> ParserResult<&str, Header> {
        context(
            "In-Reply-To header",
            map(
                tuple((
                    tag_no_case("In-Reply-To"),
                    hcolon,
                    cut(consumed(separated_list1(comma, callid))),
                )),
                |(name, separator, (value, call_ids))| {
                    Header::InReplyTo(InReplyToHeader::new(
                        GenericHeader::new(name, separator, value),
                        call_ids,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, InReplyToHeader,
    };
    use claims::assert_ok;

    valid_header!(InReplyTo, InReplyToHeader, "In-Reply-To");
    header_equality!(InReplyTo, "In-Reply-To");
    header_inequality!(InReplyTo, "In-Reply-To");

    #[test]
    fn test_valid_in_reply_to_header_with_a_single_call_id() {
        valid_header("In-Reply-To: 70710@saturn.bell-tel.com", |header| {
            assert_eq!(header.call_ids().len(), 1);
            assert_eq!(
                header.call_ids().first().unwrap(),
                "70710@saturn.bell-tel.com"
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
                    "70710@saturn.bell-tel.com"
                );
                assert_eq!(
                    header.call_ids().last().unwrap(),
                    "17320@saturn.bell-tel.com"
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
        let header = Header::try_from(
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
