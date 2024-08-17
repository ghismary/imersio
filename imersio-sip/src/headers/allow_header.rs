//! SIP Allow header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{Method, Methods};

/// Representation of an Allow header.
///
/// The Allow header field lists the set of methods supported by the UA
/// generating the message.
///
/// [[RFC3261, Section 20.5](https://datatracker.ietf.org/doc/html/rfc3261#section-20.5)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct AllowHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    methods: Methods,
}

impl AllowHeader {
    pub(crate) fn new(header: GenericHeader, methods: Vec<Method>) -> Self {
        Self {
            header,
            methods: methods.into(),
        }
    }

    /// Get a reference to the list of methods from the Allow header.
    pub fn methods(&self) -> &Methods {
        &self.methods
    }

    /// Tell whether Allow header contains the given method.
    pub fn contains(&self, method: Method) -> bool {
        self.methods.iter().any(|m| m == method)
    }
}

impl HeaderAccessor for AllowHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Allow")
    }
    fn normalized_value(&self) -> String {
        self.methods.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::method::parser::method;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{AllowHeader, Header};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list0,
        sequence::tuple,
    };

    pub(crate) fn allow(input: &str) -> ParserResult<&str, Header> {
        context(
            "Allow header",
            map(
                tuple((
                    tag_no_case("Allow"),
                    hcolon,
                    cut(consumed(separated_list0(comma, method))),
                )),
                |(name, separator, (value, methods))| {
                    Header::Allow(AllowHeader::new(
                        GenericHeader::new(name, separator, value),
                        methods,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::headers::{
        tests::{header_equality, header_inequality, valid_header},
        HeaderAccessor,
    };
    use crate::{AllowHeader, Header, Method};
    use claims::assert_ok;

    valid_header!(Allow, AllowHeader, "Allow");
    header_equality!(Allow, "Allow");
    header_inequality!(Allow, "Allow");

    #[test]
    fn test_valid_allow_header_with_methods() {
        valid_header("Allow: INVITE, ACK, OPTIONS, CANCEL, BYE", |header| {
            assert!(!header.methods().is_empty());
            assert_eq!(header.methods().len(), 5);
            assert!(header.contains(Method::INVITE));
            assert!(header.contains(Method::ACK));
            assert!(header.contains(Method::OPTIONS));
            assert!(header.contains(Method::CANCEL));
            assert!(header.contains(Method::BYE));
            assert!(!header.contains(Method::REGISTER));
        });
    }

    #[test]
    fn test_valid_allow_header_empty() {
        valid_header("Allow:", |header| {
            assert!(header.methods().is_empty());
            assert_eq!(header.methods().len(), 0);
            assert!(!header.contains(Method::INVITE));
            assert!(!header.contains(Method::REGISTER));
        });
    }

    #[test]
    fn test_valid_allow_header_empty_with_space_characters() {
        valid_header("Allow:      ", |header| {
            assert!(header.methods().is_empty());
            assert_eq!(header.methods().len(), 0);
            assert!(!header.contains(Method::CANCEL));
            assert!(!header.contains(Method::BYE));
        });
    }

    #[test]
    fn test_allow_header_equality_same_headers_with_space_characters_differences() {
        header_equality(
            "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE",
            "Allow:   INVITE, ACK,  OPTIONS, CANCEL,     BYE",
        );
    }

    #[test]
    fn test_allow_header_equality_with_different_methods_order() {
        header_equality(
            "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE",
            "Allow: INVITE, BYE, CANCEL, OPTIONS, ACK",
        );
    }

    #[test]
    fn test_allow_header_inequality_with_different_methods() {
        header_inequality("Allow: INVITE", "Allow: BYE");
    }

    #[test]
    fn test_allow_header_inequality_with_first_header_having_more_methods_than_the_second() {
        header_inequality(
            "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE",
            "Allow: BYE, CANCEL, REGISTER, ACK",
        );
    }

    #[test]
    fn test_allow_header_inequality_with_first_header_having_less_methods_than_the_second() {
        header_inequality("Allow: INVITE, ACK", "Allow: INVITE, BYE, CANCEL, ACK");
    }

    #[test]
    fn test_allow_header_inequality_with_non_uppercase_methods() {
        header_inequality(
            "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE",
            "allow: invite, Bye, CanCeL, OptionS, acK",
        );
    }

    #[test]
    fn test_allow_header_to_string() {
        let header = Header::try_from("allow:   INVITE , ACK,  OPTIONS   , CANCEL,     BYE");
        if let Header::Allow(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "allow:   INVITE , ACK,  OPTIONS   , CANCEL,     BYE"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE"
            );
            assert_eq!(
                header.to_compact_string(),
                "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE"
            );
        }
    }
}
