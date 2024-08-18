//! SIP Accept-Encoding header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{AcceptEncoding, AcceptEncodings};

/// Representation of an Accept-Encoding header.
///
/// The Accept-Encoding header field is similar to Accept, but restricts the
/// content-codings that are acceptable in the response.
///
/// [[RFC3261, Section 20.2](https://datatracker.ietf.org/doc/html/rfc3261#section-20.2)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct AcceptEncodingHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    encodings: AcceptEncodings,
}

impl AcceptEncodingHeader {
    pub(crate) fn new(header: GenericHeader, encodings: Vec<AcceptEncoding>) -> Self {
        Self {
            header,
            encodings: encodings.into(),
        }
    }

    /// Get a reference to the encodings of the `Accept-Encoding` header.
    pub fn encodings(&self) -> &AcceptEncodings {
        &self.encodings
    }
}

impl HeaderAccessor for AcceptEncodingHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Accept-Encoding")
    }
    fn normalized_value(&self) -> String {
        self.encodings.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::accept_encoding::parser::encoding;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{AcceptEncodingHeader, Header, TokenString};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list0,
        sequence::tuple,
    };

    pub(crate) fn accept_encoding(input: &str) -> ParserResult<&str, Header> {
        context(
            "Accept-Encoding header",
            map(
                tuple((
                    map(tag_no_case("Accept-Encoding"), TokenString::new),
                    hcolon,
                    cut(consumed(separated_list0(comma, encoding))),
                )),
                |(name, separator, (value, encodings))| {
                    Header::AcceptEncoding(AcceptEncodingHeader::new(
                        GenericHeader::new(name, separator, value),
                        encodings,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::headers::{
        tests::{header_equality, header_inequality, invalid_header, valid_header},
        HeaderAccessor,
    };
    use crate::{AcceptEncodingHeader, Header};
    use claims::assert_ok;

    valid_header!(AcceptEncoding, AcceptEncodingHeader, "Accept-Encoding");
    header_equality!(AcceptEncoding, "Accept-Encoding");
    header_inequality!(AcceptEncoding, "Accept-Encoding");

    #[test]
    fn test_valid_accept_encoding_header_with_single_encoding() {
        valid_header("Accept-Encoding: gzip", |header| {
            assert!(!header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 1);
            assert!(header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(!header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_with_several_encodings() {
        valid_header("Accept-Encoding: gzip, deflate", |header| {
            assert!(!header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 2);
            assert!(header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_with_several_encodings_and_space_characters() {
        valid_header("Accept-Encoding: gzip    ,compress,  deflate", |header| {
            assert!(!header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 3);
            assert!(header.encodings().contains("gzip"));
            assert!(header.encodings().contains("compress"));
            assert!(header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_empty() {
        valid_header("Accept-Encoding:", |header| {
            assert!(header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 0);
            assert!(!header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(!header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_empty_with_space_characters() {
        valid_header("Accept-Encoding:     ", |header| {
            assert!(header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 0);
            assert!(!header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(!header.encodings().contains("deflate"));
        });
    }

    #[test]
    fn test_valid_accept_encoding_header_with_parameter() {
        valid_header("Accept-Encoding: deflate, gzip;q=1.0", |header| {
            assert!(!header.encodings().is_empty());
            assert_eq!(header.encodings().len(), 2);
            assert!(header.encodings().contains("gzip"));
            assert!(!header.encodings().contains("compress"));
            assert!(header.encodings().contains("deflate"));
            let gzip_encoding = header.encodings().get("gzip").unwrap();
            assert_eq!(gzip_encoding.parameters().len(), 1);
            assert_eq!(gzip_encoding.parameters().first().unwrap().key(), "q");
            assert_eq!(
                gzip_encoding.parameters().first().unwrap().value(),
                Some("1.0")
            );
            let gzip_q = gzip_encoding.q();
            assert!(gzip_q.is_some());
            assert!((gzip_q.unwrap() - 1.0).abs() < 0.01);
        });
    }

    #[test]
    fn test_invalid_accept_encoding_header_with_invalid_character() {
        invalid_header("Accept-Encoding: 😁");
    }

    #[test]
    fn test_accept_encoding_header_equality_with_space_characters_differences() {
        header_equality("Accept-Encoding: gzip", "Accept-Encoding:  gzip");
    }

    #[test]
    fn test_accept_encoding_header_equality_with_different_encodings_order() {
        header_equality(
            "Accept-Encoding: gzip, deflate",
            "Accept-Encoding: deflate, gzip",
        );
    }

    #[test]
    fn test_accept_encoding_header_inequality_with_different_encodings() {
        header_inequality("Accept-Encoding: gzip", "Accept-Encoding: deflate");
    }

    #[test]
    fn test_accept_encoding_header_inequality_with_first_header_having_more_encodings_than_the_second(
    ) {
        header_inequality("Accept-Encoding: gzip, deflate", "Accept-Encoding: deflate");
    }

    #[test]
    fn test_accept_encoding_header_inequality_with_first_header_having_less_encodings_than_the_second(
    ) {
        header_inequality("Accept-Encoding: deflate", "Accept-Encoding: gzip, deflate");
    }

    #[test]
    fn test_accept_encoding_header_to_string() {
        let header = Header::try_from("accept-encoding:   gZip  , DeFlate");
        if let Header::AcceptEncoding(header) = header.unwrap() {
            assert_eq!(header.to_string(), "accept-encoding:   gZip  , DeFlate");
            assert_eq!(
                header.to_normalized_string(),
                "Accept-Encoding: gzip, deflate"
            );
            assert_eq!(header.to_compact_string(), "Accept-Encoding: gzip, deflate");
        }
    }
}
