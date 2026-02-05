//! SIP Call-Info header parsing and generation.

use derive_more::Display;
use std::ops::Deref;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::utils::compare_vectors;
use crate::{CallInfo, CallInfos};

/// Representation of a Call-Info header.
///
/// The Call-Info header field provides additional information about the
/// caller or callee, depending on whether it is found in a request or
/// response.
///
/// [[RFC3261, Section 20.9](https://datatracker.ietf.org/doc/html/rfc3261#section-20.9)]
#[derive(Clone, Debug, Display, Eq)]
#[display("{}", header)]
pub struct CallInfoHeader {
    header: GenericHeader,
    infos: CallInfos,
}

impl CallInfoHeader {
    pub(crate) fn new(header: GenericHeader, infos: Vec<CallInfo>) -> Self {
        Self {
            header,
            infos: infos.into(),
        }
    }
}

impl CallInfoHeader {
    /// Get a reference to the infos from the Call-Info header.
    pub fn infos(&self) -> &CallInfos {
        &self.infos
    }
}

impl HeaderAccessor for CallInfoHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Call-Info")
    }
    fn normalized_value(&self) -> String {
        self.infos.to_string()
    }
}

impl PartialEq for CallInfoHeader {
    fn eq(&self, other: &Self) -> bool {
        compare_vectors(self.infos().deref(), other.infos().deref())
    }
}

pub(crate) mod parser {
    use nom::{
        Parser,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
    };

    use crate::{
        CallInfoHeader, Header, TokenString,
        common::call_info::parser::info,
        headers::GenericHeader,
        parser::{ParserResult, comma, hcolon},
    };

    pub(crate) fn call_info(input: &str) -> ParserResult<&str, Header> {
        context(
            "Call-Info header",
            map(
                (
                    map(tag_no_case("Call-Info"), TokenString::new),
                    hcolon,
                    cut(consumed(separated_list1(comma, info))),
                ),
                |(name, separator, (value, infos))| {
                    Header::CallInfo(CallInfoHeader::new(
                        GenericHeader::new(name, separator, value),
                        infos,
                    ))
                },
            ),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::wrapped_string::WrappedString;
    use crate::{
        CallInfoHeader, CallInfoParameter, GenericParameter, Header, TokenString, Uri,
        headers::{
            HeaderAccessor,
            tests::{header_equality, header_inequality, invalid_header, valid_header},
        },
    };
    use claims::assert_ok;

    valid_header!(CallInfo, CallInfoHeader, "Call-Info");
    header_equality!(CallInfo, "Call-Info");
    header_inequality!(CallInfo, "Call-Info");

    #[test]
    fn test_valid_call_info_header_with_icon_and_info() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info",
            |header| {
                assert_eq!(header.infos().len(), 2);
                let first_uri = Uri::try_from("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(first_uri));
                let first_call_info = header.infos().get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert_eq!(first_call_info.parameters().len(), 1);
                assert_eq!(
                    first_call_info.parameters().first().unwrap(),
                    &CallInfoParameter::IconPurpose
                );
                let second_uri = Uri::try_from("http://www.example.com/alice/").unwrap();
                let second_uri = second_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(second_uri));
                let second_call_info = header.infos().get(second_uri);
                assert!(second_call_info.is_some());
                let second_call_info = second_call_info.unwrap();
                assert_eq!(second_call_info.parameters().len(), 1);
                assert_eq!(
                    second_call_info.parameters().first().unwrap(),
                    &CallInfoParameter::InfoPurpose
                );
                let third_uri = Uri::try_from("http://www.example.com/bob/").unwrap();
                let third_uri = third_uri.as_absolute_uri().unwrap();
                assert!(!header.infos().contains(third_uri));
            },
        );
    }

    #[test]
    fn test_valid_call_info_header_with_custom_purpose() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=photo",
            |header| {
                assert_eq!(header.infos().len(), 1);
                let first_uri = Uri::try_from("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(first_uri));
                let first_call_info = header.infos().get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert_eq!(first_call_info.parameters().len(), 1);
                assert_eq!(
                    first_call_info.parameters().first().unwrap(),
                    &CallInfoParameter::OtherPurpose(TokenString::new("photo"))
                );
            },
        );
    }

    #[test]
    fn test_valid_call_info_header_with_custom_param_with_value() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;info=photo",
            |header| {
                assert_eq!(header.infos().len(), 1);
                let first_uri = Uri::try_from("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(first_uri));
                let first_call_info = header.infos().get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert_eq!(first_call_info.parameters().len(), 1);
                assert_eq!(
                    first_call_info.parameters().first().unwrap(),
                    &CallInfoParameter::Other(GenericParameter::new(
                        TokenString::new("info"),
                        Some(WrappedString::new_not_wrapped(TokenString::new("photo")))
                    ))
                );
            },
        );
    }

    #[test]
    fn test_valid_call_info_header_with_custom_param_without_value() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;info",
            |header| {
                assert_eq!(header.infos().len(), 1);
                let first_uri = Uri::try_from("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(first_uri));
                let first_call_info = header.infos().get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert_eq!(first_call_info.parameters().len(), 1);
                assert_eq!(
                    first_call_info.parameters().first().unwrap(),
                    &CallInfoParameter::Other(GenericParameter::new(
                        TokenString::new("info"),
                        None
                    ))
                );
            },
        );
    }

    #[test]
    fn test_valid_call_info_header_without_param() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg>",
            |header| {
                assert_eq!(header.infos().len(), 1);
                let first_uri = Uri::try_from("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(first_uri));
                let first_call_info = header.infos().get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert!(first_call_info.parameters().is_empty());
            },
        );
    }

    #[test]
    fn test_invalid_call_info_header_empty() {
        invalid_header("Call-Info:");
    }

    #[test]
    fn test_invalid_call_info_header_empty_with_space_characters() {
        invalid_header("Call-Info:    ");
    }

    #[test]
    fn test_invalid_call_info_header_with_invalid_character() {
        invalid_header("Call-Info: 😁");
    }

    #[test]
    fn test_invalid_call_info_header_with_invalid_uri() {
        invalid_header("Call-Info: http://wwww.example.com/alice/photo.jpg");
    }

    #[test]
    fn test_call_info_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/>;purpose=info",
            "Call-Info: <http://wwww.example.com/alice/photo.jpg>; purpose=icon, <http://www.example.com/alice/> ;purpose=info",
        );
    }

    #[test]
    fn test_call_info_header_equality_with_inverted_infos() {
        header_equality(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info",
            "Call-Info: <http://www.example.com/alice/> ;purpose=info, <http://wwww.example.com/alice/photo.jpg> ;purpose=icon",
        );
    }

    #[test]
    fn test_call_info_header_equality_with_different_cases() {
        header_equality(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon",
            "call-info: <http://wwww.example.com/alice/photo.jpg> ;puRpoSe=Icon",
        );
    }

    #[test]
    fn test_call_info_header_inequality_different_uris_with_same_purpose() {
        header_inequality(
            "Call-Info: <http://www.example.com/alice/> ;purpose=info",
            "Call-Info: <http://www.example.com/bob/> ;purpose=info",
        );
    }

    #[test]
    fn test_call_info_header_inequality_same_uri_with_different_purposes() {
        header_inequality(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon",
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=info",
        );
    }

    #[test]
    fn test_call_info_header_to_string() {
        let header = Header::try_from(
            "call-info:   <http://wwww.example.com/alice/photo.jpg> ;puRpoSe=Icon",
        );
        if let Header::CallInfo(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "call-info:   <http://wwww.example.com/alice/photo.jpg> ;puRpoSe=Icon"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Call-Info: <http://wwww.example.com/alice/photo.jpg>;purpose=icon"
            );
            assert_eq!(
                header.to_compact_string(),
                "Call-Info: <http://wwww.example.com/alice/photo.jpg>;purpose=icon"
            );
        }
    }
}
