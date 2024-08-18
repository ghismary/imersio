use crate::{SipError, TokenString};
use std::cmp::Ordering;

/// Representation of the priority from a `PriorityHeader`.
#[derive(Clone, Debug, Eq)]
pub enum Priority {
    /// The `emergency` priority.
    Emergency,
    /// The `urgent` priority.
    Urgent,
    /// The `normal` priority.
    Normal,
    /// The `non-urgent` priority.
    NonUrgent,
    /// Any other extension priority.
    Other(TokenString),
}

impl Priority {
    pub(crate) fn new(priority: TokenString) -> Self {
        match priority.to_ascii_lowercase().as_str() {
            "emergency" => Self::Emergency,
            "urgent" => Self::Urgent,
            "normal" => Self::Normal,
            "non-urgent" => Self::NonUrgent,
            value => Self::Other(TokenString::new(value)),
        }
    }
}

impl std::fmt::Display for Priority {
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

impl PartialEq for Priority {
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

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl TryFrom<&str> for Priority {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Priority::new(TokenString::try_from(value)?))
    }
}

pub(crate) mod parser {
    use crate::parser::{token, ParserResult};
    use crate::{Priority, TokenString};
    use nom::{branch::alt, bytes::complete::tag_no_case, combinator::map, error::context};

    #[inline]
    fn other_priority(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }

    pub(crate) fn priority_value(input: &str) -> ParserResult<&str, Priority> {
        context(
            "priority_value",
            map(
                alt((
                    map(tag_no_case("emergency"), TokenString::new),
                    map(tag_no_case("urgent"), TokenString::new),
                    map(tag_no_case("normal"), TokenString::new),
                    map(tag_no_case("non-urgent"), TokenString::new),
                    other_priority,
                )),
                Priority::new,
            ),
        )(input)
    }
}
