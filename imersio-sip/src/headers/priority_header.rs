//! SIP Priority header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::Priority;

/// Representation of a Priority header.
///
/// The Priority header field indicates the urgency of the request as perceived by the client. The
/// Priority header field describes the priority that the SIP request should have to the receiving
/// human or its agent. For example, it may be factored into decisions about call routing and
/// acceptance. For these decisions, a message containing no Priority header field SHOULD be
/// treated as if it specified a Priority of "normal". The Priority header field does not influence
/// the use of communications resources such as packet forwarding priority in routers or access to
/// circuits in PSTN gateways. The header field can have the values "non-urgent", "normal",
/// "urgent", and "emergency", but additional values can be defined elsewhere. It is RECOMMENDED
/// that the value of "emergency" only be used when life, limb, or property are in imminent danger.
/// Otherwise, there are no semantics defined for this header field.
///
/// [[RFC3261, Section 20.26](https://datatracker.ietf.org/doc/html/rfc3261#section-20.26)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct PriorityHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    priority: Priority,
}

impl PriorityHeader {
    pub(crate) fn new(header: GenericHeader, priority: Priority) -> Self {
        Self { header, priority }
    }

    /// Get a reference to the `Priority` of the Priority header.
    pub fn priority(&self) -> &Priority {
        &self.priority
    }
}

impl HeaderAccessor for PriorityHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Priority")
    }
    fn normalized_value(&self) -> String {
        self.priority.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::priority::parser::priority_value;
    use crate::headers::GenericHeader;
    use crate::parser::{hcolon, ParserResult};
    use crate::{Header, PriorityHeader, TokenString};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        sequence::tuple,
    };

    pub(crate) fn priority(input: &str) -> ParserResult<&str, Header> {
        context(
            "Priority header",
            map(
                tuple((
                    map(tag_no_case("Priority"), TokenString::new),
                    hcolon,
                    cut(consumed(priority_value)),
                )),
                |(name, separator, (value, priority))| {
                    Header::Priority(PriorityHeader::new(
                        GenericHeader::new(name, separator, value),
                        priority,
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
        Header, Priority, PriorityHeader, TokenString,
    };
    use claims::assert_ok;

    valid_header!(Priority, PriorityHeader, "Priority");
    header_equality!(Priority, "Priority");
    header_inequality!(Priority, "Priority");

    #[test]
    fn test_valid_priority_header_1() {
        valid_header("Priority: emergency", |header| {
            assert_eq!(header.priority(), &Priority::Emergency);
        });
    }

    #[test]
    fn test_valid_priority_header_2() {
        valid_header("Priority: non-urgent", |header| {
            assert_eq!(header.priority(), &Priority::NonUrgent);
        });
    }

    #[test]
    fn test_valid_priority_header_with_custom_priority() {
        valid_header("Priority: my-own-priority", |header| {
            assert_eq!(
                header.priority(),
                &Priority::Other(TokenString::new("my-own-priority"))
            );
        });
    }

    #[test]
    fn test_invalid_priority_header_empty() {
        invalid_header("Priority:");
    }

    #[test]
    fn test_invalid_priority_header_empty_with_space_characters() {
        invalid_header("Priority:         ");
    }

    #[test]
    fn test_invalid_priority_header_with_invalid_character() {
        invalid_header("Priority: 😁");
    }

    #[test]
    fn test_priority_header_equality_with_space_characters_differences() {
        header_equality("Priority: Normal", "Priority :    Normal");
    }

    #[test]
    fn test_priority_header_equality_with_custom_priority() {
        header_equality("Priority: my-own-priority", "Priority :   my-own-priority");
    }

    #[test]
    fn test_priority_header_inequality_with_different_predefined_values() {
        header_inequality("Priority: emergency", "Priority: non-urgent");
    }

    #[test]
    fn test_priority_header_inequality_with_different_custom_values() {
        header_inequality("Priority: my-own-priority", "Priority: your-own-priority");
    }

    #[test]
    fn test_priority_header_inequality_with_one_predefined_value_and_a_custom_one() {
        header_inequality("Priority: normal", "Priority: my-own-priority");
    }

    #[test]
    fn test_priority_header_to_string() {
        let header = Header::try_from("prIOrItY  :  EMERGENCY");
        if let Header::Priority(header) = header.unwrap() {
            assert_eq!(header.to_string(), "prIOrItY  :  EMERGENCY");
            assert_eq!(header.to_normalized_string(), "Priority: emergency");
            assert_eq!(header.to_compact_string(), "Priority: emergency");
        }
    }
}
