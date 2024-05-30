use partial_eq_refs::PartialEqRefs;

use crate::Method;

use super::{generic_header::GenericHeader, HeaderAccessor};

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
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct CSeqHeader {
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
    crate::header::generic_header_accessors!(header);

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

impl std::fmt::Display for CSeqHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq<CSeqHeader> for CSeqHeader {
    fn eq(&self, other: &CSeqHeader) -> bool {
        self.cseq == other.cseq && self.method == other.method
    }
}

#[cfg(test)]
mod tests {
    use super::CSeqHeader;
    use crate::{header::HeaderAccessor, Header};
    use claim::{assert_err, assert_ok};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(CSeqHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert_ok!(&header);
        if let Header::CSeq(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not a CSeq header");
        }
    }

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

    fn invalid_header(header: &str) {
        assert_err!(Header::from_str(header));
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

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::CSeq(first_header), Header::CSeq(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not a CSeq header");
        }
    }

    #[test]
    fn test_cseq_header_equality_same_header_with_space_characters_differences() {
        header_equality("CSeq: 4711 INVITE", "CSeq  :     4711   INVITE");
    }

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::CSeq(first_header), Header::CSeq(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not a CSeq header");
        }
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
        let header = Header::from_str("cseq  : 4711     INVITE");
        if let Header::CSeq(header) = header.unwrap() {
            assert_eq!(header.to_string(), "cseq  : 4711     INVITE");
            assert_eq!(header.to_normalized_string(), "CSeq: 4711 INVITE");
            assert_eq!(header.to_compact_string(), "CSeq: 4711 INVITE");
        }
    }
}
