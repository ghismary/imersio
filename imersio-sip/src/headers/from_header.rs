//! SIP From header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{FromParameter, FromParameters, NameAddress};

/// Representation of a From header.
///
/// The From header field indicates the initiator of the request. This may be different from the
/// initiator of the dialog. Requests sent by the callee to the caller use the callee's address in
/// the From header field.
///
/// [[RFC3261, Section 20.20](https://datatracker.ietf.org/doc/html/rfc3261#section-20.20)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display("{}", header)]
pub struct FromHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    address: NameAddress,
    parameters: FromParameters,
}

impl FromHeader {
    pub(crate) fn new(
        header: GenericHeader,
        address: NameAddress,
        parameters: Vec<FromParameter>,
    ) -> Self {
        Self {
            header,
            address,
            parameters: parameters.into(),
        }
    }

    /// Get a reference to the address from the From header.
    pub fn address(&self) -> &NameAddress {
        &self.address
    }

    /// Get a reference to the parameters from the From header.
    pub fn parameters(&self) -> &FromParameters {
        &self.parameters
    }

    /// Get the value of the `tag` parameter from the From header, if it has one.
    pub fn tag(&self) -> Option<&str> {
        self.parameters
            .iter()
            .find(|param| matches!(param, FromParameter::Tag(_)))
            .and_then(|param| param.tag())
    }
}

impl HeaderAccessor for FromHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("f")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("From")
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
    use crate::common::contact::parser::{addr_spec, name_addr};
    use crate::common::generic_parameter::parser::generic_param;
    use crate::headers::GenericHeader;
    use crate::parser::{equal, hcolon, semi, token, ParserResult};
    use crate::{FromHeader, FromParameter, GenericParameter, Header, NameAddress};
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::many0,
        sequence::{pair, preceded, separated_pair, tuple},
    };

    fn tag_param(input: &str) -> ParserResult<&str, GenericParameter> {
        context(
            "tag_param",
            map(
                separated_pair(tag_no_case("tag"), equal, token),
                |(key, value)| GenericParameter::new(key, Some(value)),
            ),
        )(input)
    }

    fn from_param(input: &str) -> ParserResult<&str, FromParameter> {
        context(
            "from_param",
            map(alt((tag_param, generic_param)), Into::into),
        )(input)
    }

    fn from_spec(input: &str) -> ParserResult<&str, (NameAddress, Vec<FromParameter>)> {
        context(
            "from_spec",
            pair(
                alt((map(addr_spec, |uri| NameAddress::new(uri, None)), name_addr)),
                many0(preceded(semi, from_param)),
            ),
        )(input)
    }

    pub(crate) fn from(input: &str) -> ParserResult<&str, Header> {
        context(
            "From header",
            map(
                tuple((
                    alt((tag_no_case("From"), tag_no_case("f"))),
                    hcolon,
                    cut(consumed(from_spec)),
                )),
                |(name, separator, (value, (address, parameters)))| {
                    Header::From(FromHeader::new(
                        GenericHeader::new(name, separator, value),
                        address,
                        parameters,
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
        FromHeader, Header, Uri,
    };
    use claims::assert_ok;

    valid_header!(From, FromHeader, "From");
    header_equality!(From, "From");
    header_inequality!(From, "From");

    #[test]
    fn test_valid_from_header_with_display_name() {
        valid_header(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            |header| {
                assert_eq!(header.address().display_name(), Some("A. G. Bell"));
                assert_eq!(
                    header.address().uri(),
                    Uri::try_from("sip:agb@bell-telephone.com").unwrap()
                );
                assert_eq!(header.parameters().len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("a48s"));
                assert_eq!(header.tag(), Some("a48s"));
            },
        );
    }

    #[test]
    fn test_valid_from_header_without_display_name() {
        valid_header(
            "From: <sip:+12125551212@server.phone2net.com>;tag=887s",
            |header| {
                assert_eq!(header.address().display_name(), None);
                assert_eq!(
                    header.address().uri(),
                    Uri::try_from("sip:+12125551212@server.phone2net.com").unwrap()
                );
                assert_eq!(header.parameters.len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("887s"));
                assert_eq!(header.tag(), Some("887s"));
            },
        )
    }

    #[test]
    fn test_valid_from_header_in_compact_form() {
        valid_header(
            "f: Anonymous <sip:c8oqz84zk7z@privacy.org>;tag=hyh8",
            |header| {
                assert_eq!(header.address().display_name(), Some("Anonymous"));
                assert_eq!(
                    header.address().uri(),
                    Uri::try_from("sip:c8oqz84zk7z@privacy.org").unwrap()
                );
                assert_eq!(header.parameters.len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("hyh8"));
                assert_eq!(header.tag(), Some("hyh8"));
            },
        )
    }

    #[test]
    fn test_invalid_from_header_empty() {
        invalid_header("From:");
    }

    #[test]
    fn test_invalid_from_header_empty_with_space_characters() {
        invalid_header("From:    ");
    }

    #[test]
    fn test_invalid_from_header_with_invalid_character() {
        invalid_header("From: 😁");
    }

    #[test]
    fn test_from_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From:    "A. G. Bell"  <sip:agb@bell-telephone.com>; tag=a48s"#,
        );
    }

    #[test]
    fn test_from_header_equality_same_header_with_different_display_names() {
        header_equality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: Bell <sip:agb@bell-telephone.com> ;tag=a48s"#,
        );
    }

    #[test]
    fn test_from_header_equality_same_header_with_different_cases() {
        header_equality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;TAG=a48s"#,
        );
    }

    #[test]
    fn test_from_header_inequality_different_uris() {
        header_inequality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: "A. G. Bell" <sip:agc@bell-telephone.com> ;tag=a48s"#,
        );
    }

    #[test]
    fn test_from_header_inequality_different_tag_parameters() {
        header_inequality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=hyh8"#,
        );
    }

    #[test]
    fn test_from_header_to_string() {
        let header = Header::try_from(
            r#"from :    "A. G. Bell"   <sip:agb@bell-telephone.com> ;   tag  = a48s"#,
        );
        if let Header::From(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"from :    "A. G. Bell"   <sip:agb@bell-telephone.com> ;   tag  = a48s"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"From: "A. G. Bell" <sip:agb@bell-telephone.com>;tag=a48s"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"f: "A. G. Bell" <sip:agb@bell-telephone.com>;tag=a48s"#
            );
        }
    }
}
