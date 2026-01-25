//! SIP Server header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{ServerValue, ServerValues};

/// Representation of a Server header.
///
/// The Server header field contains information about the software used by the UAS to handle the
/// request.
///
/// [[RFC3261, Section 20.35](https://datatracker.ietf.org/doc/html/rfc3261#section-20.35)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
pub struct ServerHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    values: ServerValues,
}

impl ServerHeader {
    pub(crate) fn new(header: GenericHeader, values: Vec<ServerValue>) -> Self {
        let values: ServerValues = values.into();
        Self {
            header,
            values: values.set_separator(" "),
        }
    }

    /// Get the list of server values from the Server header.
    pub fn values(&self) -> &ServerValues {
        &self.values
    }
}

impl HeaderAccessor for ServerHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Server")
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
        Header, ServerHeader, TokenString,
    };

    pub(crate) fn server(input: &str) -> ParserResult<&str, Header> {
        context(
            "Server header",
            map(
                (
                    map(tag_no_case("Server"), TokenString::new),
                    hcolon,
                    cut(consumed(separated_list1(lws, server_val))),
                ),
                |(name, separator, (value, values))| {
                    Header::Server(ServerHeader::new(
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
        Header, Product, ServerHeader, ServerValue, TokenString,
    };
    use claims::assert_ok;

    valid_header!(Server, ServerHeader, "Server");
    header_equality!(Server, "Server");
    header_inequality!(Server, "Server");

    #[test]
    fn test_valid_server_header_with_a_single_product() {
        valid_header("Server: HomeServer", |header| {
            assert_eq!(header.values().len(), 1);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Product(Product::new(TokenString::new("HomeServer"), None))
            );
        });
    }

    #[test]
    fn test_valid_server_header_with_a_single_product_and_version() {
        valid_header("Server: HomeServer/2", |header| {
            assert_eq!(header.values().len(), 1);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Product(Product::new(
                    TokenString::new("HomeServer"),
                    Some(TokenString::new("2"))
                ))
            );
        });
    }

    #[test]
    fn test_valid_server_header_with_a_single_comment() {
        valid_header("Server: (A comment)", |header| {
            assert_eq!(header.values().len(), 1);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Comment("A comment".to_string())
            );
        });
    }

    #[test]
    fn test_valid_server_header_with_several_products() {
        valid_header("Server: HomeServer/2 OtherServer", |header| {
            assert_eq!(header.values().len(), 2);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Product(Product::new(
                    TokenString::new("HomeServer"),
                    Some(TokenString::new("2"))
                ))
            );
            assert_eq!(
                header.values().last().unwrap(),
                &ServerValue::Product(Product::new(TokenString::new("OtherServer"), None))
            );
        });
    }

    #[test]
    fn test_valid_server_header_with_a_product_and_a_comment() {
        valid_header("Server: HomeServer (A comment)", |header| {
            assert_eq!(header.values().len(), 2);
            assert_eq!(
                header.values().first().unwrap(),
                &ServerValue::Product(Product::new(TokenString::new("HomeServer"), None))
            );
            assert_eq!(
                header.values().last().unwrap(),
                &ServerValue::Comment("A comment".to_string())
            );
        });
    }

    #[test]
    fn test_invalid_server_header_empty() {
        invalid_header("Server:");
    }

    #[test]
    fn test_invalid_server_header_empty_with_space_characters() {
        invalid_header("Server:    ");
    }

    #[test]
    fn test_invalid_server_header_with_invalid_character() {
        invalid_header("Server: 😁");
    }

    #[test]
    fn test_server_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            "Server: HomeServer/2 OtherServer",
            "Server  :       HomeServer/2               OtherServer",
        );
    }

    #[test]
    fn test_server_header_equality_same_header_with_products_in_a_different_order() {
        header_equality(
            "Server: HomeServer/2 OtherServer",
            "Server: OtherServer HomeServer/2",
        );
    }

    #[test]
    fn test_server_header_inequality_different_products() {
        header_inequality("Server: HomeServer/2", "Server: OtherServer");
    }

    #[test]
    fn test_server_header_inequality_same_product_with_different_versions() {
        header_inequality("Server: HomeServer/2", "Server: HomeServer/3");
    }

    #[test]
    fn test_server_header_inequality_with_first_header_having_more_products_than_the_second() {
        header_inequality("Server: HomeServer/2 OtherServer", "Server: HomeServer/2");
    }

    #[test]
    fn test_server_header_inequality_with_first_header_having_less_products_than_the_second() {
        header_inequality("Server: HomeServer/2", "Server: HomeServer/2 OtherServer");
    }

    #[test]
    fn test_server_header_to_string() {
        let header = Header::try_from("serVer  :      HomeServer/2                OtherServer");
        if let Header::Server(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "serVer  :      HomeServer/2                OtherServer"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Server: HomeServer/2 OtherServer"
            );
            assert_eq!(
                header.to_compact_string(),
                "Server: HomeServer/2 OtherServer"
            );
        }
    }
}
