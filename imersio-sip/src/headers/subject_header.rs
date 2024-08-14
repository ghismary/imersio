//! SIP Subject header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};

/// Representation of a Subject header.
///
/// The Subject header field provides a summary or indicates the nature of the call, allowing call
/// filtering without having to parse the session description. The session description does not have
/// to use the same subject indication as the invitation.
///
/// [[RFC3261, Section 20.36](https://datatracker.ietf.org/doc/html/rfc3261#section-20.36)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display("{}", header)]
pub struct SubjectHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    subject: String,
}

impl SubjectHeader {
    pub(crate) fn new<S: Into<String>>(header: GenericHeader, subject: S) -> Self {
        Self {
            header,
            subject: subject.into(),
        }
    }

    /// Get the subject from the Subject header.
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl HeaderAccessor for SubjectHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("s")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Subject")
    }
    fn normalized_value(&self) -> String {
        self.subject.clone()
    }
}

pub(crate) mod parser {
    use crate::headers::GenericHeader;
    use crate::parser::{hcolon, text_utf8_trim, ParserResult};
    use crate::{Header, SubjectHeader};
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map, opt},
        error::context,
        sequence::tuple,
    };

    pub(crate) fn subject(input: &str) -> ParserResult<&str, Header> {
        context(
            "Subject header",
            map(
                tuple((
                    alt((tag_no_case("Subject"), tag_no_case("s"))),
                    hcolon,
                    cut(consumed(opt(text_utf8_trim))),
                )),
                |(name, separator, (value, subject))| {
                    Header::Subject(SubjectHeader::new(
                        GenericHeader::new(name, separator, value),
                        subject.unwrap_or_default(),
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
            tests::{header_equality, header_inequality, valid_header},
            HeaderAccessor,
        },
        Header, SubjectHeader,
    };
    use claims::assert_ok;

    valid_header!(Subject, SubjectHeader, "Subject");
    header_equality!(Subject, "Subject");
    header_inequality!(Subject, "Subject");

    #[test]
    fn test_valid_subject_header() {
        valid_header("Subject: Need more boxes", |header| {
            assert_eq!(header.subject(), "Need more boxes");
        });
    }

    #[test]
    fn test_valid_subject_header_empty() {
        valid_header("Subject:", |header| {
            assert_eq!(header.subject(), "");
        });
    }

    #[test]
    fn test_valid_subject_header_empty_with_space_characters() {
        valid_header("Subject:    ", |header| {
            assert_eq!(header.subject(), "");
        });
    }

    #[test]
    fn test_valid_subject_header_with_utf8_character() {
        valid_header("Subject: 😁", |header| {
            assert_eq!(header.subject(), "😁");
        });
    }

    #[test]
    fn test_valid_subject_header_in_compact_form() {
        valid_header("s: Tech Support", |header| {
            assert_eq!(header.subject(), "Tech Support");
        });
    }

    #[test]
    fn test_subject_header_equality_same_header_with_trailing_space_characters_differences() {
        header_equality("Subject: Need more boxes", "Subject: Need more boxes    ");
    }

    #[test]
    fn test_subject_header_inequality_different_values() {
        header_inequality("Subject: Need more boxes", "s: Tech Support");
    }

    #[test]
    fn test_subject_header_to_string() {
        let header = Header::try_from("subJecT    :              Need more boxes");
        if let Header::Subject(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "subJecT    :              Need more boxes"
            );
            assert_eq!(header.to_normalized_string(), "Subject: Need more boxes");
            assert_eq!(header.to_compact_string(), "s: Need more boxes");
        }
    }
}
