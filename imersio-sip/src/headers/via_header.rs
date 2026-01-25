//! SIP Via header parsing and generation.

use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{Via, Vias};

/// Representation of a Via header.
///
/// The Via header field indicates the path taken by the request so far and indicates the path that
/// should be followed in routing responses. The branch ID parameter in the Via header field values
/// serves as a transaction identifier and is used by proxies to detect loops.
///
/// [[RFC3261, Section 20.42](https://datatracker.ietf.org/doc/html/rfc3261#section-20.42)]
#[derive(Clone, Debug, Eq, derive_more::Display, PartialEqExtras)]
#[display("{}", header)]
pub struct ViaHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    vias: Vias,
}

impl ViaHeader {
    pub(crate) fn new(header: GenericHeader, vias: Vec<Via>) -> Self {
        Self {
            header,
            vias: vias.into(),
        }
    }

    /// Get a reference to the vias from the Via header.
    pub fn vias(&self) -> &Vias {
        &self.vias
    }
}

impl HeaderAccessor for ViaHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("v")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Via")
    }
    fn normalized_value(&self) -> String {
        self.vias.to_string()
    }
}

pub(crate) mod parser {
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        Parser,
    };

    use crate::{
        common::via::parser::via_parm,
        headers::GenericHeader,
        parser::{comma, hcolon, ParserResult},
        Header, TokenString, ViaHeader,
    };

    pub(crate) fn via(input: &str) -> ParserResult<&str, Header> {
        context(
            "Via header",
            map(
                (
                    map(
                        alt((tag_no_case("Via"), tag_no_case("v"))),
                        TokenString::new,
                    ),
                    hcolon,
                    cut(consumed(separated_list1(comma, via_parm))),
                ),
                |(name, separator, (value, vias))| {
                    Header::Via(ViaHeader::new(
                        GenericHeader::new(name, separator, value),
                        vias,
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
        Header, Host, HostnameString, Protocol, TokenString, Transport, ViaHeader,
    };
    use claims::assert_ok;
    use std::net::{IpAddr, Ipv4Addr};

    valid_header!(Via, ViaHeader, "Via");
    header_equality!(Via, "Via");
    header_inequality!(Via, "Via");

    #[test]
    fn test_valid_via_header() {
        valid_header(
            "Via: SIP/2.0/UDP erlang.bell-telephone.com:5060;branch=z9hG4bK87asdks7",
            |header| {
                assert_eq!(header.vias().len(), 1);
                let first_via = header.vias().first().unwrap();
                assert_eq!(
                    first_via.protocol(),
                    &Protocol::new(
                        TokenString::new("SIP"),
                        TokenString::new("2.0"),
                        Transport::Udp
                    )
                );
                assert_eq!(
                    first_via.host(),
                    &Host::Name(HostnameString::try_from("erlang.bell-telephone.com").unwrap())
                );
                assert_eq!(first_via.port(), Some(5060));
                assert_eq!(
                    first_via.parameters().first().unwrap().branch(),
                    Some("z9hG4bK87asdks7".to_string())
                );
            },
        );
    }

    #[test]
    fn test_valid_via_header_with_several_parameters() {
        valid_header(
            "Via: SIP/2.0/UDP 192.0.2.1:5060 ;received=192.0.2.207 ;branch=z9hG4bK77asjd",
            |header| {
                assert_eq!(header.vias().len(), 1);
                let first_via = header.vias().first().unwrap();
                assert_eq!(
                    first_via.protocol(),
                    &Protocol::new(
                        TokenString::new("SIP"),
                        TokenString::new("2.0"),
                        Transport::Udp
                    )
                );
                assert_eq!(
                    first_via.host(),
                    &Host::Ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)))
                );
                assert_eq!(first_via.port(), Some(5060));
                assert_eq!(
                    first_via.parameters().first().unwrap().received(),
                    Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 207)))
                );
                assert_eq!(
                    first_via.parameters().last().unwrap().branch(),
                    Some("z9hG4bK77asjd".to_string())
                )
            },
        );
    }

    #[test]
    fn test_valid_via_header_in_compact_form() {
        valid_header(
            "v: SIP / 2.0 / UDP first.example.com: 4000;ttl=16;maddr=224.2.0.1 ;branch=z9hG4bKa7c6a8dlze.1",
            |header| {
                assert_eq!(header.vias().len(), 1);
                let first_via = header.vias().first().unwrap();
                assert_eq!(
                    first_via.protocol(),
                    &Protocol::new(TokenString::new("SIP"), TokenString::new("2.0"), Transport::Udp)
                );
                assert_eq!(
                    first_via.host(),
                    &Host::Name(HostnameString::try_from("first.example.com").unwrap())
                );
                assert_eq!(first_via.port(), Some(4000));
                let mut it = first_via.parameters().iter();
                let param = it.next();
                assert_eq!(
                    param.unwrap().ttl(),
                    Some(16)
                );
                let param = it.next();
                assert_eq!(param.unwrap().maddr(), Some(Host::Ip(IpAddr::V4(Ipv4Addr::new(224, 2, 0, 1)))));
                let param = it.next();
                assert_eq!(
                    param.unwrap().branch(),
                    Some("z9hG4bKa7c6a8dlze.1".to_string())
                )
            },
        );
    }

    #[test]
    fn test_invalid_via_header_empty() {
        invalid_header("Via:");
    }

    #[test]
    fn test_invalid_via_header_empty_with_space_characters() {
        invalid_header("Via:    ");
    }

    #[test]
    fn test_invalid_via_header_with_invalid_character() {
        invalid_header("Via: 😁");
    }

    #[test]
    fn test_via_equality_same_header_with_space_characters_differences() {
        header_equality(
            "Via: SIP/2.0/UDP erlang.bell-telephone.com:5060;branch=z9hG4bK87asdks7",
            "Via :    SIP/2.0/UDP       erlang.bell-telephone.com:5060 ;  branch=z9hG4bK87asdks7",
        );
    }

    #[test]
    fn test_via_header_equality_same_parameters_in_a_different_order() {
        header_equality(
            "Via: SIP/2.0/UDP 192.0.2.1:5060 ;received=192.0.2.207 ;branch=z9hG4bK77asjd",
            "Via: SIP/2.0/UDP 192.0.2.1:5060 ;branch=z9hG4bK77asjd;received=192.0.2.207",
        );
    }

    #[test]
    fn test_via_header_equality_same_vias_with_different_cases() {
        header_equality(
            "v: SIP / 2.0 / UDP first.example.com: 4000;ttl=16;maddr=224.2.0.1 ;branch=z9hG4bKa7c6a8dlze.1", 
            "V: sip / 2.0 / Udp first.example.com: 4000;TTL=16;MAddr=224.2.0.1 ;brAnch=z9hG4bKa7c6a8dlze.1"
        );
    }

    #[test]
    fn test_via_header_inequality_with_different_vias() {
        header_inequality(
            "Via: SIP/2.0/UDP erlang.bell-telephone.com:5060;branch=z9hG4bK87asdks7",
            "Via: SIP/2.0/UDP 192.0.2.1:5060 ;received=192.0.2.207 ;branch=z9hG4bK77asjd",
        );
    }

    #[test]
    fn test_via_inequality_with_first_having_more_parameters_than_the_second() {
        header_inequality(
            "Via: SIP/2.0/UDP 192.0.2.1:5060 ;received=192.0.2.207 ;branch=z9hG4bK77asjd",
            "Via: SIP/2.0/UDP 192.0.2.1:5060 ;received=192.0.2.207",
        );
    }

    #[test]
    fn test_via_header_inequality_with_first_having_less_parameters_than_the_second() {
        header_inequality(
            "Via: SIP/2.0/UDP 192.0.2.1:5060 ;received=192.0.2.207",
            "Via: SIP/2.0/UDP 192.0.2.1:5060 ;received=192.0.2.207 ;branch=z9hG4bK77asjd",
        );
    }

    #[test]
    fn test_via_header_to_string() {
        let header = Header::try_from("via :    SIP/2.0/UDP      192.0.2.1:5060 ;ReceiveD= 192.0.2.207 ; branCH=z9hG4bK77asjd");
        if let Header::Via(header) = header.unwrap() {
            assert_eq!(header.to_string(), "via :    SIP/2.0/UDP      192.0.2.1:5060 ;ReceiveD= 192.0.2.207 ; branCH=z9hG4bK77asjd");
            assert_eq!(
                header.to_normalized_string(),
                "Via: SIP/2.0/UDP 192.0.2.1:5060;received=192.0.2.207;branch=z9hG4bK77asjd"
            );
            assert_eq!(
                header.to_compact_string(),
                "v: SIP/2.0/UDP 192.0.2.1:5060;received=192.0.2.207;branch=z9hG4bK77asjd"
            );
        }
    }
}
