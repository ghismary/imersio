//! SIP User-Agent header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{ServerValue, ServerValues};

/// Representation of a User-Agent header.
///
/// The User-Agent header field contains information about the UAC originating the request.
///
/// Revealing the specific software version of the user agent might allow the user agent to become
/// more vulnerable to attacks against software that is known to contain security holes.
/// Implementers SHOULD make the User-Agent header field a configurable option.
///
/// [[RFC3261, Section 20.41](https://datatracker.ietf.org/doc/html/rfc3261#section-20.41)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
pub struct UserAgentHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    values: ServerValues,
}

impl UserAgentHeader {
    pub(crate) fn new(header: GenericHeader, values: Vec<ServerValue>) -> Self {
        let values: ServerValues = values.into();
        Self {
            header,
            values: values.set_separator(" "),
        }
    }

    /// Get the list of server values from the User-Agent header.
    pub fn values(&self) -> &ServerValues {
        &self.values
    }
}

impl HeaderAccessor for UserAgentHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("User-Agent")
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
        common::server_value::parser::server_val,
        headers::GenericHeader,
        parser::{hcolon, lws, ParserResult},
        Header, TokenString, UserAgentHeader,
    };

    pub(crate) fn user_agent(input: &str) -> ParserResult<&str, Header> {
        context(
            "User-Agent header",
            map(
                (
                    map(tag_no_case("User-Agent"), TokenString::new),
                    hcolon,
                    cut(consumed(separated_list1(lws, server_val))),
                ),
                |(name, separator, (value, values))| {
                    Header::UserAgent(UserAgentHeader::new(
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
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, Product, ServerValue, TokenString, UserAgentHeader,
    };
    use claims::assert_ok;

    valid_header!(UserAgent, UserAgentHeader, "User-Agent");
    header_equality!(UserAgent, "User-Agent");
    header_inequality!(UserAgent, "User-Agent");

    #[test]
    fn test_valid_user_agent_header_with_a_single_product() {
        valid_header("User-Agent: Softphone", |header| {
            assert_eq!(header.values().len(), 1);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Product(Product::new(TokenString::new("Softphone"), None))
            );
        });
    }

    #[test]
    fn test_valid_user_agent_header_with_a_single_product_and_version() {
        valid_header("User-Agent: Softphone/Beta1.5", |header| {
            assert_eq!(header.values().len(), 1);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Product(Product::new(
                    TokenString::new("Softphone"),
                    Some(TokenString::new("Beta1.5"))
                ))
            );
        });
    }

    #[test]
    fn test_valid_user_agent_header_with_a_single_comment() {
        valid_header("User-Agent: (A comment)", |header| {
            assert_eq!(header.values().len(), 1);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Comment("A comment".to_string())
            );
        });
    }

    #[test]
    fn test_valid_user_agent_header_with_several_products() {
        valid_header("User-Agent: Softphone/Beta1.5 OtherProduct", |header| {
            assert_eq!(header.values().len(), 2);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Product(Product::new(
                    TokenString::new("Softphone"),
                    Some(TokenString::new("Beta1.5"))
                ))
            );
            assert_eq!(
                header.values().last().unwrap(),
                &ServerValue::Product(Product::new(TokenString::new("OtherProduct"), None))
            );
        });
    }

    #[test]
    fn test_valid_user_agent_header_with_a_product_and_a_comment() {
        valid_header("User-Agent: Softphone (A comment)", |header| {
            assert_eq!(header.values().len(), 2);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Product(Product::new(TokenString::new("Softphone"), None))
            );
            assert_eq!(
                header.values().last().unwrap(),
                &ServerValue::Comment("A comment".to_string())
            );
        });
    }

    #[test]
    fn test_invalid_user_agent_header_empty() {
        invalid_header("User-Agent:");
    }

    #[test]
    fn test_invalid_user_agent_header_empty_with_space_characters() {
        invalid_header("User-Agent:    ");
    }

    #[test]
    fn test_invalid_user_agent_header_with_invalid_character() {
        invalid_header("User-Agent: 😁");
    }

    #[test]
    fn test_user_agent_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            "User-Agent: Softphone/Beta1.5 OtherProduct",
            "User-Agent  :       Softphone / Beta1.5               OtherProduct",
        );
    }

    #[test]
    fn test_user_agent_header_equality_same_header_with_products_in_a_different_order() {
        header_equality(
            "User-Agent: Softphone/Beta1.5 OtherProduct",
            "User-Agent: OtherProduct Softphone / Beta1.5",
        );
    }

    #[test]
    fn test_user_agent_header_inequality_different_products() {
        header_inequality("User-Agent: Softphone/Beta1.5", "User-Agent: OtherProduct");
    }

    #[test]
    fn test_user_agent_header_inequality_same_product_with_different_versions() {
        header_inequality(
            "User-Agent: Softphone/Beta1.5",
            "User-Agent: Softphone/Alpha2.3",
        );
    }

    #[test]
    fn test_user_agent_header_inequality_with_first_header_having_more_products_than_the_second() {
        header_inequality(
            "User-Agent: Softphone/Beta1.5 OtherProduct",
            "User-Agent: Softphone/Beta1.5",
        );
    }

    #[test]
    fn test_user_agent_header_inequality_with_first_header_having_less_products_than_the_second() {
        header_inequality(
            "User-Agent: Softphone/Beta1.5",
            "User-Agent: Softphone/Beta1.5 OtherProduct",
        );
    }

    #[test]
    fn test_user_agent_header_to_string() {
        let header =
            Header::try_from("user-AgenT  :      Softphone /  Beta1.5                OtherProduct");
        if let Header::UserAgent(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "user-AgenT  :      Softphone /  Beta1.5                OtherProduct"
            );
            assert_eq!(
                header.to_normalized_string(),
                "User-Agent: Softphone/Beta1.5 OtherProduct"
            );
            assert_eq!(
                header.to_compact_string(),
                "User-Agent: Softphone/Beta1.5 OtherProduct"
            );
        }
    }
}
