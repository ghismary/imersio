//! SIP CSeq header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::Method;

/// Representation of a CSeq header.
///
/// A CSeq header field in a request contains a single decimal sequence number
/// and the request method. The sequence number MUST be expressible as a
/// 32-bit unsigned integer. The method part of CSeq is case-sensitive. The
/// CSeq header field serves to order transactions within a dialog, to provide
/// a means to uniquely identify transactions, and to differentiate between
/// new requests and request retransmissions.
///
/// [[RFC3261, Section 20.16](https://datatracker.ietf.org/doc/html/rfc3261#section-20.16)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display("{}", header)]
pub struct CSeqHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    cseq: u32,
    method: Method,
}

impl CSeqHeader {
    pub(crate) fn new(header: GenericHeader, cseq: u32, method: Method) -> Self {
        Self {
            header,
            cseq,
            method,
        }
    }

    /// Get the cseq from the CSeq header.
    pub fn cseq(&self) -> u32 {
        self.cseq
    }

    /// Get the method from the CSeq header.
    pub fn method(&self) -> &Method {
        &self.method
    }
}

impl HeaderAccessor for CSeqHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("CSeq")
    }
    fn normalized_value(&self) -> String {
        format!("{} {}", self.cseq, self.method)
    }
}

pub(crate) mod parser {
    use crate::common::method::parser::method;
    use crate::headers::GenericHeader;
    use crate::parser::{digit, hcolon, lws, ParserResult};
    use crate::{CSeqHeader, Header};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map, recognize},
        error::context,
        multi::many1,
        sequence::{separated_pair, tuple},
    };

    pub(crate) fn cseq(input: &str) -> ParserResult<&str, Header> {
        context(
            "CSeq header",
            map(
                tuple((
                    tag_no_case("CSeq"),
                    hcolon,
                    cut(consumed(separated_pair(
                        map(recognize(many1(digit)), |cseq| cseq.parse::<u32>().unwrap()),
                        lws,
                        method,
                    ))),
                )),
                |(name, separator, (value, (cseq, method)))| {
                    Header::CSeq(CSeqHeader::new(
                        GenericHeader::new(name, separator, value),
                        cseq,
                        method,
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
        CSeqHeader, Header,
    };
    use claims::assert_ok;

    valid_header!(CSeq, CSeqHeader, "CSeq");
    header_equality!(CSeq, "CSeq");
    header_inequality!(CSeq, "CSeq");

    #[test]
    fn test_valid_cseq_header_1() {
        valid_header("CSeq: 4711 INVITE", |header| {
            assert_eq!(header.cseq(), 4711);
            assert_eq!(header.method(), "INVITE");
        });
    }

    #[test]
    fn test_valid_cseq_header_2() {
        valid_header("CSeq: 89378 ACK", |header| {
            assert_eq!(header.cseq(), 89_378);
            assert_eq!(header.method(), "ACK");
        });
    }

    #[test]
    fn test_invalid_cseq_header_empty() {
        invalid_header("CSeq:");
    }

    #[test]
    fn test_invalid_cseq_header_empty_with_space_characters() {
        invalid_header("CSeq:    ");
    }

    #[test]
    fn test_invalid_cseq_header_with_invalid_character() {
        invalid_header("CSeq: 😁");
    }

    #[test]
    fn test_invalid_cseq_header_with_missing_method() {
        invalid_header("CSeq: 4711");
    }

    #[test]
    fn test_invalid_cseq_header_with_missing_sequence_number() {
        invalid_header("CSeq: INVITE");
    }

    #[test]
    fn test_cseq_header_equality_same_header_with_space_characters_differences() {
        header_equality("CSeq: 4711 INVITE", "CSeq  :     4711   INVITE");
    }

    #[test]
    fn test_cseq_header_inequality_() {
        header_inequality("CSeq: 4711 INVITE", "CSeq: 173 ACK");
    }

    #[test]
    fn test_cseq_header_inequality_different_sequence_numbers() {
        header_inequality("CSeq: 4711 INVITE", "CSeq: 173 INVITE");
    }

    #[test]
    fn test_cseq_header_inequality_different_methods() {
        header_inequality("CSeq: 4711 INVITE", "CSeq: 4711 ACK");
    }

    #[test]
    fn test_cseq_header_to_string() {
        let header = Header::try_from("cseq  : 4711     INVITE");
        if let Header::CSeq(header) = header.unwrap() {
            assert_eq!(header.to_string(), "cseq  : 4711     INVITE");
            assert_eq!(header.to_normalized_string(), "CSeq: 4711 INVITE");
            assert_eq!(header.to_compact_string(), "CSeq: 4711 INVITE");
        }
    }
}
