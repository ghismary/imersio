//! Parsing and generation of the host part of a SIP uri.

use derive_more::IsVariant;
use std::hash::Hash;
use std::net::IpAddr;

/// Representation of a host part of a SIP URI.
#[derive(Clone, Debug, Eq, IsVariant)]
pub enum Host {
    /// A hostname
    Name(String),
    /// An Ip address.
    Ip(IpAddr),
}

impl Host {
    /// Get the name of the `Host` is it is one.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Name(name) => Some(name.as_str()),
            _ => None,
        }
    }

    /// Get the ip of the `Host` if it is one.
    pub fn ip(&self) -> Option<&IpAddr> {
        match self {
            Self::Ip(ip) => Some(ip),
            _ => None,
        }
    }
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Name(name) => name.clone(),
                Self::Ip(ip) => ip.to_string(),
            }
        )
    }
}

impl Default for Host {
    fn default() -> Self {
        Host::Name("localhost".to_string())
    }
}

impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Name(a), Self::Name(b)) => a.eq_ignore_ascii_case(b),
            (Self::Ip(a), Self::Ip(b)) => a == b,
            _ => false,
        }
    }
}

impl Hash for Host {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Name(name) => name.to_ascii_lowercase().hash(state),
            Self::Ip(ip) => ip.hash(state),
        }
    }
}

pub(crate) mod parser {
    use crate::parser::{digit, hex_digit, take1, ParserResult};
    use crate::Host;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, map_res, opt, recognize, verify},
        error::context,
        multi::{many0, many1, many_m_n},
        sequence::{delimited, pair, preceded, tuple},
    };
    use std::net::IpAddr;

    fn is_valid_hostname(input: &str) -> bool {
        let mut labels: Vec<&str> = input.split('.').collect();
        // A valid hostname may end by '.', if this is the case the last label
        // will be empty, and so we remove before further processing.
        if labels.last().is_some_and(|label| label.is_empty()) {
            labels.pop();
        }
        // If nothing remains, this is not valid.
        if labels.is_empty() {
            return false;
        }
        // All other labels must not be empty.
        if labels.iter().any(|label| label.is_empty()) {
            return false;
        }
        // The '-' must not be located at the beginning or at the end of a
        // label.
        if labels
            .iter()
            .all(|label| label.starts_with('-') || label.ends_with('-'))
        {
            return false;
        }
        labels
            .pop()
            .is_some_and(|label| label.as_bytes()[0].is_ascii_alphabetic())
    }

    fn hostname(input: &str) -> ParserResult<&str, Host> {
        context(
            "hostname",
            map(
                verify(
                    recognize(many1(verify(take1, |c| {
                        c.is_ascii_alphanumeric() || "-.".contains(*c)
                    }))),
                    is_valid_hostname,
                ),
                |name| Host::Name(name.into()),
            ),
        )(input)
    }

    #[inline]
    fn is_valid_ipv4_address_number(input: &str) -> bool {
        input.parse::<u8>().is_ok()
    }

    fn ipv4_address_number(input: &str) -> ParserResult<&str, &str> {
        recognize(many_m_n(1, 3, digit))(input)
    }

    pub(crate) fn ipv4_address(input: &str) -> ParserResult<&str, IpAddr> {
        context(
            "ipv4_address",
            map_res(
                recognize(tuple((
                    verify(ipv4_address_number, is_valid_ipv4_address_number),
                    tag("."),
                    verify(ipv4_address_number, is_valid_ipv4_address_number),
                    tag("."),
                    verify(ipv4_address_number, is_valid_ipv4_address_number),
                    tag("."),
                    verify(ipv4_address_number, is_valid_ipv4_address_number),
                ))),
                |ipv4| ipv4.parse(),
            ),
        )(input)
    }

    fn hex4(input: &str) -> ParserResult<&str, &str> {
        recognize(many_m_n(1, 4, hex_digit))(input)
    }

    fn hexseq(input: &str) -> ParserResult<&str, &str> {
        recognize(pair(hex4, many0(pair(tag(":"), hex4))))(input)
    }

    fn hexpart(input: &str) -> ParserResult<&str, &str> {
        recognize(alt((
            hexseq,
            recognize(tuple((hexseq, tag("::"), hexseq))),
            recognize(pair(tag("::"), hexseq)),
        )))(input)
    }

    pub(crate) fn ipv6_address(input: &str) -> ParserResult<&str, IpAddr> {
        context(
            "ipv6_address",
            map_res(
                recognize(pair(hexpart, opt(pair(tag(":"), ipv4_address)))),
                |ipv6| ipv6.parse(),
            ),
        )(input)
    }

    fn ipv6_reference(input: &str) -> ParserResult<&str, IpAddr> {
        context(
            "ipv6_reference",
            delimited(tag("["), ipv6_address, tag("]")),
        )(input)
    }

    pub(crate) fn host(input: &str) -> ParserResult<&str, Host> {
        context(
            "host",
            alt((
                hostname,
                map(ipv4_address, Host::Ip),
                map(ipv6_reference, Host::Ip),
            )),
        )(input)
    }

    pub(crate) fn port(input: &str) -> ParserResult<&str, u16> {
        context(
            "port",
            map_res(recognize(many_m_n(1, 5, digit)), |digits| digits.parse()),
        )(input)
    }

    pub(crate) fn hostport(input: &str) -> ParserResult<&str, (Host, Option<u16>)> {
        context("hostport", pair(host, opt(preceded(tag(":"), port))))(input)
    }
}
