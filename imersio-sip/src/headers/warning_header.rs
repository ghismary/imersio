//! SIP Warning header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{WarningValue, WarningValues};

/// Representation of a Warning header.
///
/// The Warning header field is used to carry additional information about the status of a response.
/// Warning header field values are sent with responses and contain a three-digit warning code, host
/// name, and warning text.
///
/// The "warn-text" should be in a natural language that is most likely to be intelligible to the
/// human user receiving the response. This decision can be based on any available knowledge, such
/// as the location of the user, the Accept-Language field in a request, or the Content-Language
/// field in a response.
///
/// [[RFC3261, Section 20.43](https://datatracker.ietf.org/doc/html/rfc3261#section-20.43)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
pub struct WarningHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    values: WarningValues,
}

impl WarningHeader {
    pub(crate) fn new(header: GenericHeader, values: Vec<WarningValue>) -> Self {
        Self {
            header,
            values: values.into(),
        }
    }

    /// Get the list of warning values from the Warning header.
    pub fn values(&self) -> &WarningValues {
        &self.values
    }
}

impl HeaderAccessor for WarningHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Warning")
    }
    fn normalized_value(&self) -> String {
        self.values.to_string()
    }
}

pub(crate) mod parser {
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        Parser,
    };

    use crate::{
        common::warning_value::parser::warning_value,
        headers::GenericHeader,
        parser::{comma, hcolon, ParserResult},
        Header, TokenString, WarningHeader,
    };

    pub(crate) fn warning(input: &str) -> ParserResult<&str, Header> {
        context(
            "Warning header",
            map(
                (
                    map(tag_no_case("Warning"), TokenString::new),
                    hcolon,
                    cut(consumed(separated_list1(comma, warning_value))),
                ),
                |(name, separator, (value, values))| {
                    Header::Warning(WarningHeader::new(
                        GenericHeader::new(name, separator, value),
                        values,
                    ))
                },
            ),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::warn_code::WarnCode;
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, WarnAgent, WarningHeader,
    };
    use claims::assert_ok;

    valid_header!(Warning, WarningHeader, "Warning");
    header_equality!(Warning, "Warning");
    header_inequality!(Warning, "Warning");

    #[test]
    fn test_valid_warning_header_with_a_single_warning() {
        valid_header(
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood""#,
            |header| {
                assert_eq!(header.values().len(), 1);
                let first_warning = header.values().first().unwrap();
                assert_eq!(
                    first_warning.code(),
                    WarnCode::SESSION_DESCRIPTION_PARAMETER_NOT_UNDERSTOOD
                );
                assert_eq!(
                    first_warning.agent(),
                    &WarnAgent::try_from("isi.edu").unwrap()
                );
                assert_eq!(
                    first_warning.text(),
                    "Session parameter 'foo' not understood"
                );
            },
        );
    }

    #[test]
    fn test_valid_warning_header_with_a_single_warning_2() {
        valid_header(
            r#"Warning: 301 isi.edu "Incompatible network address type 'E.164'""#,
            |header| {
                assert_eq!(header.values().len(), 1);
                let first_warning = header.values().first().unwrap();
                assert_eq!(
                    first_warning.code(),
                    WarnCode::INCOMPATIBLE_NETWORK_ADDRESS_FORMATS
                );
                assert_eq!(
                    first_warning.agent(),
                    &WarnAgent::try_from("isi.edu").unwrap()
                );
                assert_eq!(
                    first_warning.text(),
                    "Incompatible network address type 'E.164'"
                );
            },
        );
    }

    #[test]
    fn test_valid_warning_header_with_several_warnings() {
        valid_header(
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood", 301 isi.edu "Incompatible network address type 'E.164'""#,
            |header| {
                assert_eq!(header.values().len(), 2);
                let first_warning = header.values().first().unwrap();
                assert_eq!(
                    first_warning.code(),
                    WarnCode::SESSION_DESCRIPTION_PARAMETER_NOT_UNDERSTOOD
                );
                assert_eq!(
                    first_warning.agent(),
                    &WarnAgent::try_from("isi.edu").unwrap()
                );
                assert_eq!(
                    first_warning.text(),
                    "Session parameter 'foo' not understood"
                );
                let second_warning = header.values().last().unwrap();
                assert_eq!(
                    second_warning.code(),
                    WarnCode::INCOMPATIBLE_NETWORK_ADDRESS_FORMATS
                );
                assert_eq!(
                    first_warning.agent(),
                    &WarnAgent::try_from("isi.edu").unwrap()
                );
                assert_eq!(
                    second_warning.text(),
                    "Incompatible network address type 'E.164'"
                );
            },
        );
    }

    #[test]
    fn test_invalid_warning_header_empty() {
        invalid_header("Warning:");
    }

    #[test]
    fn test_invalid_warning_header_empty_with_space_characters() {
        invalid_header("Warning:    ");
    }

    #[test]
    fn test_invalid_warning_header_with_invalid_character() {
        invalid_header("Warning: 😁");
    }

    #[test]
    fn test_warning_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood""#,
            r#"Warning :    307 isi.edu "Session parameter 'foo' not understood""#,
        );
    }

    #[test]
    fn test_warning_header_equality_same_header_with_warnings_in_a_different_order() {
        header_equality(
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood", 301 isi.edu "Incompatible network address type 'E.164'""#,
            r#"Warning: 301 isi.edu "Incompatible network address type 'E.164'", 307 isi.edu "Session parameter 'foo' not understood""#,
        );
    }

    #[test]
    fn test_warning_header_equality_same_warning_with_different_texts() {
        header_equality(
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood""#,
            r#"Warning: 307 isi.edu "Session parameter 'bar' not understood""#,
        );
    }

    #[test]
    fn test_warning_header_inequality_different_warnings() {
        header_inequality(
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood""#,
            r#"Warning: 301 isi.edu "Incompatible network address type 'E.164'""#,
        );
    }

    #[test]
    fn test_warning_header_inequality_with_first_header_having_more_warnings_than_the_second() {
        header_inequality(
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood", 301 isi.edu "Incompatible network address type 'E.164'""#,
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood""#,
        );
    }

    #[test]
    fn test_warning_header_inequality_with_first_header_having_less_warnings_than_the_second() {
        header_inequality(
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood""#,
            r#"Warning: 307 isi.edu "Session parameter 'foo' not understood", 301 isi.edu "Incompatible network address type 'E.164'""#,
        );
    }

    #[test]
    fn test_warning_header_to_string() {
        let header = Header::try_from(
            r#"warNING    :            307 isi.edu "Session parameter 'foo' not understood""#,
        );
        if let Header::Warning(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"warNING    :            307 isi.edu "Session parameter 'foo' not understood""#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Warning: 307 isi.edu "Session parameter 'foo' not understood""#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Warning: 307 isi.edu "Session parameter 'foo' not understood""#
            );
        }
    }
}
