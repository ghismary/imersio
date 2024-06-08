use partial_eq_refs::PartialEqRefs;

use crate::HeaderAccessor;

use super::generic_header::GenericHeader;

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
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct PriorityHeader {
    header: GenericHeader,
    priority: PriorityValue,
}

impl PriorityHeader {
    pub(crate) fn new(header: GenericHeader, priority: PriorityValue) -> Self {
        Self { header, priority }
    }

    /// Get a reference to the `Priority` of the Priority header.
    pub fn priority(&self) -> &PriorityValue {
        &self.priority
    }
}

impl HeaderAccessor for PriorityHeader {
    crate::header::generic_header_accessors!(header);

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

impl std::fmt::Display for PriorityHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for PriorityHeader {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

/// Representation of the priority from a `PriorityHeader`.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub enum PriorityValue {
    /// The `emergency` priority.
    Emergency,
    /// The `urgent` priority.
    Urgent,
    /// The `normal` priority.
    Normal,
    /// The `non-urgent` priority.
    NonUrgent,
    /// Any other extension priority.
    Other(String),
}

impl std::fmt::Display for PriorityValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Emergency => "emergency",
                Self::Urgent => "urgent",
                Self::Normal => "normal",
                Self::NonUrgent => "non-urgent",
                Self::Other(value) => &value,
            }
        )
    }
}

impl PartialEq for PriorityValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Emergency, Self::Emergency)
            | (Self::Urgent, Self::Urgent)
            | (Self::Normal, Self::Normal)
            | (Self::NonUrgent, Self::NonUrgent) => true,
            (Self::Other(a), Self::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PriorityHeader;
    use crate::header::PriorityValue;
    use crate::{
        header::tests::{header_equality, header_inequality, invalid_header, valid_header},
        Header, HeaderAccessor,
    };
    use claims::assert_ok;
    use std::str::FromStr;

    valid_header!(Priority, PriorityHeader, "Priority");
    header_equality!(Priority, "Priority");
    header_inequality!(Priority, "Priority");

    #[test]
    fn test_valid_priority_header_1() {
        valid_header("Priority: emergency", |header| {
            assert_eq!(header.priority(), PriorityValue::Emergency);
        });
    }

    #[test]
    fn test_valid_priority_header_2() {
        valid_header("Priority: non-urgent", |header| {
            assert_eq!(header.priority(), PriorityValue::NonUrgent);
        });
    }

    #[test]
    fn test_valid_priority_header_with_custom_priority() {
        valid_header("Priority: my-own-priority", |header| {
            assert_eq!(
                header.priority(),
                PriorityValue::Other("my-own-priority".to_string())
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
        let header = Header::from_str("prIOrItY  :  EMERGENCY");
        if let Header::Priority(header) = header.unwrap() {
            assert_eq!(header.to_string(), "prIOrItY  :  EMERGENCY");
            assert_eq!(header.to_normalized_string(), "Priority: emergency");
            assert_eq!(header.to_compact_string(), "Priority: emergency");
        }
    }
}
