//! SIP Reply-To header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{GenericParameter, GenericParameters, NameAddress, TokenString};

/// Representation of a Reply-To header.
///
/// The Reply-To header field contains a logical return URI that may be different from the From
/// header field. For example, the URI MAY be used to return missed calls or unestablished sessions.
/// If the user wished to remain anonymous, the header field SHOULD either be omitted from the
/// request or populated in such a way that does not reveal any private information.
///
/// [[RFC3261, Section 20.31](https://datatracker.ietf.org/doc/html/rfc3261#section-20.31)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
pub struct ReplyToHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    address: NameAddress,
    parameters: GenericParameters<TokenString>,
}

impl ReplyToHeader {
    pub(crate) fn new(
        header: GenericHeader,
        address: NameAddress,
        parameters: Vec<GenericParameter<TokenString>>,
    ) -> Self {
        Self {
            header,
            address,
            parameters: parameters.into(),
        }
    }

    /// Get a reference to the address from the Reply-To header.
    pub fn address(&self) -> &NameAddress {
        &self.address
    }

    /// Get a reference to the parameters from the Reply-To header.
    pub fn parameters(&self) -> &GenericParameters<TokenString> {
        &self.parameters
    }
}

impl HeaderAccessor for ReplyToHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Reply-To")
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
        sequence::{pair, preceded},
        Parser,
    };

    use crate::{
        common::{
            contact::parser::{addr_spec, name_addr},
            generic_parameter::parser::generic_param,
        },
        headers::GenericHeader,
        parser::{hcolon, semi, ParserResult},
        GenericParameter, Header, NameAddress, ReplyToHeader, TokenString,
    };

    #[inline]
    fn rplyto_param(input: &str) -> ParserResult<&str, GenericParameter<TokenString>> {
        generic_param(input)
    }

    fn rplyto_spec(
        input: &str,
    ) -> ParserResult<&str, (NameAddress, Vec<GenericParameter<TokenString>>)> {
        context(
            "rplyto_spec",
            pair(
                alt((map(addr_spec, |uri| NameAddress::new(uri, None)), name_addr)),
                many0(preceded(semi, rplyto_param)),
            ),
        )
        .parse(input)
    }

    pub(crate) fn reply_to(input: &str) -> ParserResult<&str, Header> {
        context(
            "Reply-To header",
            map(
                (
                    map(tag_no_case("Reply-To"), TokenString::new),
                    hcolon,
                    cut(consumed(rplyto_spec)),
                ),
                |(name, separator, (value, (address, parameters)))| {
                    Header::ReplyTo(ReplyToHeader::new(
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
        Header, ReplyToHeader, Uri,
    };
    use claims::assert_ok;

    valid_header!(ReplyTo, ReplyToHeader, "Reply-To");
    header_equality!(ReplyTo, "Reply-To");
    header_inequality!(ReplyTo, "Reply-To");

    #[test]
    fn test_valid_reply_to_header_with_display_name_and_without_params() {
        valid_header(r#"Reply-To: Bob <sip:bob@biloxi.com>"#, |header| {
            assert_eq!(header.address().display_name(), Some("Bob"));
            assert_eq!(
                header.address().uri(),
                Uri::try_from("sip:bob@biloxi.com").unwrap()
            );
            assert_eq!(header.parameters().len(), 0);
        });
    }

    #[test]
    fn test_valid_reply_to_header_with_display_name_and_params() {
        valid_header(
            r#"Reply-To: Bob <sip:bob@biloxi.com>;myparam=anything"#,
            |header| {
                assert_eq!(header.address().display_name(), Some("Bob"));
                assert_eq!(
                    header.address().uri(),
                    Uri::try_from("sip:bob@biloxi.com").unwrap()
                );
                assert_eq!(header.parameters().len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "myparam");
                assert_eq!(first_parameter.value(), Some("anything"));
            },
        );
    }

    #[test]
    fn test_valid_reply_to_header_without_display_name_but_with_params() {
        valid_header(
            r#"Reply-To: <sip:bob@biloxi.com>;myparam=anything"#,
            |header| {
                assert_eq!(header.address().display_name(), None);
                assert_eq!(
                    header.address().uri(),
                    Uri::try_from("sip:bob@biloxi.com").unwrap()
                );
                assert_eq!(header.parameters().len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "myparam");
                assert_eq!(first_parameter.value(), Some("anything"));
            },
        );
    }

    #[test]
    fn test_valid_reply_to_header_with_nameaddr_without_display_name() {
        valid_header("Reply-To: <sip:bob@biloxi.com>", |header| {
            assert_eq!(header.address().display_name(), None);
            assert_eq!(
                header.address().uri(),
                Uri::try_from("sip:bob@biloxi.com").unwrap()
            );
            assert_eq!(header.parameters.len(), 0);
        })
    }

    #[test]
    fn test_valid_reply_to_header_with_addrspec() {
        valid_header("Reply-To: sip:bob@biloxi.com", |header| {
            assert_eq!(header.address().display_name(), None);
            assert_eq!(
                header.address().uri(),
                Uri::try_from("sip:bob@biloxi.com").unwrap()
            );
            assert_eq!(header.parameters.len(), 0);
        })
    }

    #[test]
    fn test_invalid_reply_to_header_empty() {
        invalid_header("Reply-To:");
    }

    #[test]
    fn test_invalid_reply_to_header_empty_with_space_characters() {
        invalid_header("Reply-To:    ");
    }

    #[test]
    fn test_invalid_reply_to_header_with_invalid_character() {
        invalid_header("Reply-To: 😁");
    }

    #[test]
    fn test_reply_to_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            r#"Reply-To: Bob <sip:bob@biloxi.com>"#,
            r#"Reply-To:     Bob   <sip:bob@biloxi.com>"#,
        );
    }

    #[test]
    fn test_reply_to_header_equality_same_header_with_different_display_names() {
        header_equality(
            r#"Reply-To: Bob <sip:bob@biloxi.com>"#,
            r#"Reply-To: Alice <sip:bob@biloxi.com>"#,
        );
    }

    #[test]
    fn test_reply_to_header_equality_same_header_with_different_cases() {
        header_equality(
            r#"Reply-To: Bob <sip:bob@biloxi.com>"#,
            r#"ReplY-tO: Bob <sip:bob@biloxi.com>"#,
        );
    }

    #[test]
    fn test_reply_to_header_inequality_different_uris() {
        header_inequality(
            r#"Reply-To: Bob <sip:bob@biloxi.com>"#,
            r#"Reply-To: Bob <sip:bob@bell-telephone.com>"#,
        );
    }

    #[test]
    fn test_reply_to_header_inequality_different_parameters() {
        header_inequality(
            r#"Reply-To: Bob <sip:bob@biloxi.com>;myparam=anything"#,
            r#"Reply-To: Bob <sip:bob@biloxi.com>;myparam=something"#,
        );
    }

    #[test]
    fn test_reply_to_header_inequality_with_first_having_more_parameters_than_the_second() {
        header_inequality(
            "Reply-To: Bob <sip:bob@biloxi.com>;myparam=anything;otherparam=something",
            "Reply-To: Bob <sip:bob@biloxi.com>;myparam=anything",
        );
    }

    #[test]
    fn test_reply_to_header_inequality_with_first_having_less_languages_than_the_second() {
        header_inequality(
            "Reply-To: Bob <sip:bob@biloxi.com>;myparam=anything",
            "Reply-To: Bob <sip:bob@biloxi.com>;myparam=anything;otherparam=something",
        );
    }

    #[test]
    fn test_reply_to_header_to_string() {
        let header = Header::try_from(r#"reply-to:      Bob  <sip:bob@biloxi.com>"#);
        if let Header::ReplyTo(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"reply-to:      Bob  <sip:bob@biloxi.com>"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Reply-To: Bob <sip:bob@biloxi.com>"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Reply-To: Bob <sip:bob@biloxi.com>"#
            );
        }
    }
}
