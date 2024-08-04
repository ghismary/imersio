use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;

/// Representation of the priority from a `PriorityHeader`.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
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
    Other(String),
}

impl Priority {
    pub(crate) fn new<S: Into<String>>(priority: S) -> Self {
        match priority.into().to_ascii_lowercase().as_str() {
            "emergency" => Self::Emergency,
            "urgent" => Self::Urgent,
            "normal" => Self::Normal,
            "non-urgent" => Self::NonUrgent,
            value => Self::Other(value.into()),
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

impl From<&str> for Priority {
    fn from(value: &str) -> Self {
        Priority::new(value)
    }
}