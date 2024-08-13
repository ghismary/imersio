use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::common::warn_code::WarnCode;
use crate::WarnAgent;

/// Representation of the list of warning values in a `Warning` header.
///
/// This is usable as an iterator.
pub type WarningValues = ValueCollection<WarningValue>;

/// Representation of a warning value contained in a `Warning` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct WarningValue {
    code: WarnCode,
    agent: WarnAgent,
    text: String,
}

impl WarningValue {
    pub(crate) fn new<S: Into<String>>(code: WarnCode, agent: WarnAgent, text: S) -> Self {
        WarningValue {
            code,
            agent,
            text: text.into(),
        }
    }

    /// Get a reference to the code of the warning.
    pub fn code(&self) -> &WarnCode {
        &self.code
    }

    /// Get a reference to the agent of the warning.
    pub fn agent(&self) -> &WarnAgent {
        &self.agent
    }

    /// Get the text of the warning.
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl std::fmt::Display for WarningValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"{} {} "{}""#, self.code(), self.agent(), self.text())
    }
}

impl PartialEq for WarningValue {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.agent == other.agent
    }
}

impl Hash for WarningValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code().hash(state);
        self.agent().hash(state);
    }
}

pub(crate) mod parser {
    use crate::common::warn_agent::parser::warn_agent;
    use crate::common::warn_code::parser::warn_code;
    use crate::common::wrapped_string::WrappedString;
    use crate::parser::{quoted_string, sp, ParserResult};
    use crate::WarningValue;
    use nom::{combinator::map, error::context, sequence::tuple};

    pub(crate) fn warning_value(input: &str) -> ParserResult<&str, WarningValue> {
        context(
            "warning_value",
            map(
                tuple((warn_code, sp, warn_agent, sp, warn_text)),
                |(code, _, agent, _, text)| WarningValue::new(code, agent, text.value()),
            ),
        )(input)
    }

    #[inline]
    fn warn_text(input: &str) -> ParserResult<&str, WrappedString> {
        quoted_string(input)
    }
}
