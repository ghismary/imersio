//! SIP Retry-After header parsing and generation.

use chrono::TimeDelta;
use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use itertools::join;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::RetryParameter;

/// Representation of a Retry-After header.
///
/// The Retry-After header field can be used with a 500 (Server Internal Error) or 503 (Service
/// Unavailable) response to indicate how long the service is expected to be unavailable to the
/// requesting client and with a 404 (Not Found), 413 (Request Entity Too Large), 480 (Temporarily
/// Unavailable), 486 (Busy Here), 600 (Busy), or 603 (Decline) response to indicate when the called
/// party anticipates being available again. The value of this field is a positive integer number of
/// seconds (in decimal) after the time of the response.
///
/// [[RFC3261, Section 20.33](https://datatracker.ietf.org/doc/html/rfc3261#section-20.33)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct RetryAfterHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    retry_after: TimeDelta,
    comment: Option<String>,
    parameters: Vec<RetryParameter>,
}

impl RetryAfterHeader {
    pub(crate) fn new<S: Into<String>>(
        header: GenericHeader,
        retry_after: TimeDelta,
        comment: Option<S>,
        parameters: Vec<RetryParameter>,
    ) -> Self {
        Self {
            header,
            retry_after,
            comment: comment.map(Into::into),
            parameters,
        }
    }

    /// Get the retry after value from the Retry-After header.
    pub fn retry_after(&self) -> TimeDelta {
        self.retry_after
    }

    /// Get the comment from the Retry-After header.
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    /// Get a reference to the parameters from the Retry-After header.
    pub fn parameters(&self) -> &Vec<RetryParameter> {
        &self.parameters
    }
}

impl HeaderAccessor for RetryAfterHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Retry-After")
    }
    fn normalized_value(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.retry_after().num_seconds(),
            if self.comment().is_some() { " " } else { "" },
            self.comment().unwrap_or_default(),
            if self.parameters().is_empty() {
                ""
            } else {
                ";"
            },
            join(&self.parameters, ";")
        )
    }
}

pub(crate) mod parser {
    use crate::common::contact_parameter::parser::delta_seconds;
    use crate::common::retry_parameter::parser::retry_param;
    use crate::headers::GenericHeader;
    use crate::parser::{comment, hcolon, semi, ParserResult};
    use crate::{Header, RetryAfterHeader};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::opt,
        combinator::{consumed, cut, map},
        error::context,
        multi::many0,
        sequence::{preceded, tuple},
    };

    pub(crate) fn retry_after(input: &str) -> ParserResult<&str, Header> {
        context(
            "Retry-After header",
            map(
                tuple((
                    tag_no_case("Retry-After"),
                    hcolon,
                    cut(consumed(tuple((
                        delta_seconds,
                        opt(comment),
                        many0(preceded(semi, retry_param)),
                    )))),
                )),
                |(name, separator, (value, (retry_after, comment, params)))| {
                    Header::RetryAfter(RetryAfterHeader::new(
                        GenericHeader::new(name, separator, value),
                        retry_after,
                        comment,
                        params,
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
        Header, RetryAfterHeader,
    };
    use chrono::TimeDelta;
    use claims::assert_ok;

    valid_header!(RetryAfter, RetryAfterHeader, "Retry-After");
    header_equality!(RetryAfter, "Retry-After");
    header_inequality!(RetryAfter, "Retry-After");

    #[test]
    fn test_valid_retry_after_header_simple() {
        valid_header("Retry-After: 18000", |header| {
            assert_eq!(header.retry_after(), TimeDelta::seconds(18000));
            assert_eq!(header.comment(), None);
            assert_eq!(header.parameters().len(), 0);
        });
    }

    #[test]
    fn test_valid_retry_after_header_with_value_too_big() {
        valid_header("Retry-After: 4294968000", |header| {
            assert_eq!(header.retry_after(), TimeDelta::seconds(u32::MAX as i64));
        });
    }

    #[test]
    fn test_valid_retry_after_header_with_param() {
        valid_header("Retry-After: 18000;duration=3600", |header| {
            assert_eq!(header.retry_after(), TimeDelta::seconds(18000));
            assert_eq!(header.comment(), None);
            assert_eq!(header.parameters().len(), 1);
            assert_eq!(header.parameters().first().unwrap().key(), "duration");
            assert_eq!(header.parameters().first().unwrap().value(), Some("3600"));
        })
    }

    #[test]
    fn test_valid_retry_after_header_with_comment() {
        valid_header("Retry-After: 120 (I'm in a meeting)", |header| {
            assert_eq!(header.retry_after(), TimeDelta::seconds(120));
            assert_eq!(header.comment(), Some("I'm in a meeting"));
            assert_eq!(header.parameters().len(), 0);
        })
    }

    #[test]
    fn test_invalid_retry_after_header_empty() {
        invalid_header("Retry-After:");
    }

    #[test]
    fn test_invalid_retry_after_header_empty_with_space_characters() {
        invalid_header("Retry-After:    ");
    }

    #[test]
    fn test_invalid_retry_after_header_with_invalid_character() {
        invalid_header("Retry-After: 😁");
    }

    #[test]
    fn test_retry_after_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            "Retry-After: 18000;duration=3600",
            "Retry-After :     18000 ; duration= 3600",
        );
    }

    #[test]
    fn test_retry_after_header_inequality_different_values() {
        header_inequality("Retry-After: 18000", "Retry-After: 1800");
    }

    #[test]
    fn test_retry_after_header_inequality_same_values_but_different_parameters() {
        header_inequality(
            "Retry-After: 18000;duration=3600",
            "Retry-After: 1800;duration=180",
        );
    }

    #[test]
    fn test_retry_after_header_to_string() {
        let header = Header::try_from("retrY-aFteR  :     3600 ; Duration=180");
        if let Header::RetryAfter(header) = header.unwrap() {
            assert_eq!(header.to_string(), "retrY-aFteR  :     3600 ; Duration=180");
            assert_eq!(
                header.to_normalized_string(),
                "Retry-After: 3600;duration=180"
            );
            assert_eq!(header.to_compact_string(), "Retry-After: 3600;duration=180");
        }
    }
}
