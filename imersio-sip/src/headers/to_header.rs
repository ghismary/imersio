//! SIP To header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{NameAddress, ToParameter, ToParameters};

/// Representation of a To header.
///
/// The To header field specifies the logical recipient of the request.
///
/// The optional "display-name" is meant to be rendered by a human-user interface. The "tag"
/// parameter serves as a general mechanism for dialog identification.
///
/// [[RFC3261, Section 20.39](https://datatracker.ietf.org/doc/html/rfc3261#section-20.39)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
pub struct ToHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    address: NameAddress,
    parameters: ToParameters,
}

impl ToHeader {
    pub(crate) fn new(
        header: GenericHeader,
        address: NameAddress,
        parameters: Vec<ToParameter>,
    ) -> Self {
        Self {
            header,
            address,
            parameters: parameters.into(),
        }
    }

    /// Get a reference to the address from the To header.
    pub fn address(&self) -> &NameAddress {
        &self.address
    }

    /// Get a reference to the parameters from the To header.
    pub fn parameters(&self) -> &ToParameters {
        &self.parameters
    }

    /// Get the value of the `tag` parameter from the To header if it has one.
    pub fn tag(&self) -> Option<&str> {
        self.parameters
            .iter()
            .find(|param| matches!(param, ToParameter::Tag(_)))
            .and_then(|param| param.tag())
    }
}

impl HeaderAccessor for ToHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("t")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("To")
    }
    fn normalized_value(&self) -> String {
        format!(
            "{}{}{}",
            self.address,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
        )
    }
}

pub(crate) mod parser {
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::many0,
        sequence::{pair, preceded, separated_pair},
        Parser,
    };

    use crate::{
        common::{
            contact::parser::{addr_spec, name_addr},
            generic_parameter::parser::generic_param,
            wrapped_string::WrappedString,
        },
        headers::GenericHeader,
        parser::{equal, hcolon, semi, token, ParserResult},
        GenericParameter, Header, NameAddress, ToHeader, ToParameter, TokenString,
    };

    fn tag_param(input: &str) -> ParserResult<&str, GenericParameter<TokenString>> {
        context(
            "tag_param",
            map(
                separated_pair(map(tag_no_case("tag"), TokenString::new), equal, token),
                |(key, value)| {
                    GenericParameter::new(key, Some(WrappedString::new_not_wrapped(value)))
                },
            ),
        )
        .parse(input)
    }

    fn to_param(input: &str) -> ParserResult<&str, ToParameter> {
        context("to_param", map(alt((tag_param, generic_param)), Into::into)).parse(input)
    }

    fn to_spec(input: &str) -> ParserResult<&str, (NameAddress, Vec<ToParameter>)> {
        context(
            "to_spec",
            pair(
                alt((map(addr_spec, |uri| NameAddress::new(uri, None)), name_addr)),
                many0(preceded(semi, to_param)),
            ),
        )
        .parse(input)
    }

    pub(crate) fn to(input: &str) -> ParserResult<&str, Header> {
        context(
            "To header",
            map(
                (
                    map(alt((tag_no_case("To"), tag_no_case("t"))), TokenString::new),
                    hcolon,
                    cut(consumed(to_spec)),
                ),
                |(name, separator, (value, (address, parameters)))| {
                    Header::To(ToHeader::new(
                        GenericHeader::new(name, separator, value),
                        address,
                        parameters,
                    ))
                },
            ),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, ToHeader, Uri,
    };
    use claims::assert_ok;

    valid_header!(To, ToHeader, "To");
    header_equality!(To, "To");
    header_inequality!(To, "To");

    #[test]
    fn test_valid_to_header_with_display_name() {
        valid_header(
            r#"To: The Operator <sip:operator@cs.columbia.edu>;tag=287447"#,
            |header| {
                assert_eq!(header.address().display_name(), Some("The Operator"));
                assert_eq!(
                    header.address().uri(),
                    Uri::try_from("sip:operator@cs.columbia.edu").unwrap()
                );
                assert_eq!(header.parameters().len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("287447"));
                assert_eq!(header.tag(), Some("287447"));
            },
        );
    }

    #[test]
    fn test_valid_to_header_without_display_name() {
        valid_header("To: <sip:operator@cs.columbia.edu>;tag=287447", |header| {
            assert_eq!(header.address().display_name(), None);
            assert_eq!(
                header.address().uri(),
                Uri::try_from("sip:operator@cs.columbia.edu").unwrap()
            );
            assert_eq!(header.parameters.len(), 1);
            let first_parameter = header.parameters().first().unwrap();
            assert_eq!(first_parameter.key(), "tag");
            assert_eq!(first_parameter.value(), Some("287447"));
            assert_eq!(header.tag(), Some("287447"));
        })
    }

    #[test]
    fn test_valid_to_header_in_compact_form() {
        valid_header("t: sip:+12125551212@server.phone2net.com", |header| {
            assert_eq!(header.address().display_name(), None);
            assert_eq!(
                header.address().uri(),
                Uri::try_from("sip:+12125551212@server.phone2net.com").unwrap()
            );
            assert_eq!(header.parameters.len(), 0);
            assert_eq!(header.tag(), None);
        })
    }

    #[test]
    fn test_invalid_to_header_empty() {
        invalid_header("To:");
    }

    #[test]
    fn test_invalid_to_header_empty_with_space_characters() {
        invalid_header("To:    ");
    }

    #[test]
    fn test_invalid_to_header_with_invalid_character() {
        invalid_header("To: 😁");
    }

    #[test]
    fn test_to_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            r#"To: The Operator <sip:operator@cs.columbia.edu>;tag=287447"#,
            r#"To:    The Operator     <sip:operator@cs.columbia.edu>  ; tag=287447"#,
        );
    }

    #[test]
    fn test_to_header_equality_same_header_with_different_display_names() {
        header_equality(
            r#"To: The Operator <sip:operator@cs.columbia.edu>;tag=287447"#,
            r#"To: "Opopop" <sip:operator@cs.columbia.edu>;tag=287447"#,
        );
    }

    #[test]
    fn test_to_header_equality_same_header_with_different_cases() {
        header_equality(
            r#"To: The Operator <sip:operator@cs.columbia.edu>;tag=287447"#,
            r#"To: The Operator <sip:operator@cs.columbia.edu>;TaG=287447"#,
        );
    }

    #[test]
    fn test_to_header_inequality_different_uris() {
        header_inequality(
            r#"To: The Operator <sip:operator@cs.columbia.edu>;tag=287447"#,
            r#"To: The Operator <sip:op@cs.columbia.edu>;tag=287447"#,
        );
    }

    #[test]
    fn test_to_header_inequality_different_tag_parameters() {
        header_inequality(
            r#"To: The Operator <sip:operator@cs.columbia.edu>;tag=287447"#,
            r#"To: The Operator <sip:operator@cs.columbia.edu>;tag=hyh8"#,
        );
    }

    #[test]
    fn test_to_header_to_string() {
        let header = Header::try_from(
            r#"to:       The Operator   <sip:operator@cs.columbia.edu>  ;   tAg=  287447"#,
        );
        if let Header::To(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"to:       The Operator   <sip:operator@cs.columbia.edu>  ;   tAg=  287447"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"To: The Operator <sip:operator@cs.columbia.edu>;tag=287447"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"t: The Operator <sip:operator@cs.columbia.edu>;tag=287447"#
            );
        }
    }
}
