//! Parsing and generation of the host part of a SIP uri.

use nom_language::error::convert_error;
use std::hash::Hash;
use std::net::IpAddr;

use crate::uris::host::parser::{host, hostname};
use crate::SipError;

/// Representation of a hostname value accepting only the valid characters.
#[derive(Clone, Debug, Eq, Hash, PartialEq, derive_more::Deref, derive_more::Display)]
pub struct HostnameString(String);

impl HostnameString {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl TryFrom<&str> for HostnameString {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match hostname(value) {
            Ok((rest, hostname)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(hostname)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidHostname(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidHostname(format!(
                "Incomplete hostname `{}`",
                value
            ))),
        }
    }
}

/// Representation of a host part of a SIP URI.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum Host {
    /// A hostname
    Name(HostnameString),
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
                Self::Name(name) => name.as_str().to_string(),
                Self::Ip(ip) => {
                    match ip {
                        IpAddr::V4(ip) => ip.to_string(),
                        IpAddr::V6(ip) => format!("[{}]", ip),
                    }
                }
            }
        )
    }
}

impl Default for Host {
    fn default() -> Self {
        Host::Name(HostnameString::new("localhost"))
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

impl TryFrom<&str> for Host {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match host(value) {
            Ok((rest, host)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(host)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidHost(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidHost(format!(
                "Incomplete host `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, map_res, opt, recognize, verify},
        error::context,
        multi::{many1, many_m_n},
        sequence::{delimited, pair, preceded},
        Parser,
    };
    use std::net::IpAddr;

    use crate::{
        parser::{digit, take1, ParserResult},
        uris::host::HostnameString,
        Host,
    };

    #[inline]
    fn is_valid_hostname(input: &str) -> bool {
        let mut labels: Vec<&str> = input.split('.').collect();
        // A valid hostname may end by '.'. If this is the case, the last label
        // will be empty, and so we remove it before further processing.
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
            .any(|label| label.starts_with('-') || label.ends_with('-'))
        {
            return false;
        }
        labels
            .pop()
            .is_some_and(|label| label.as_bytes()[0].is_ascii_alphabetic())
    }

    pub(super) fn hostname(input: &str) -> ParserResult<&str, HostnameString> {
        context(
            "hostname",
            map(
                verify(
                    recognize(many1(verify(take1, |c| {
                        c.is_ascii_alphanumeric() || "-.".contains(*c)
                    }))),
                    is_valid_hostname,
                ),
                HostnameString::new,
            ),
        )
        .parse(input)
    }

    #[inline]
    fn ipv4_char(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| c.is_ascii_digit() || ".".contains(*c)).parse(input)
    }

    pub(crate) fn ipv4_address(input: &str) -> ParserResult<&str, IpAddr> {
        context(
            "ipv4_address",
            map_res(recognize(many1(ipv4_char)), |ipv4| ipv4.parse()),
        )
        .parse(input)
    }

    #[inline]
    fn ipv6_char(input: &str) -> ParserResult<&str, char> {
        verify(take1, |c| c.is_ascii_hexdigit() || ":.".contains(*c)).parse(input)
    }

    pub(crate) fn ipv6_address(input: &str) -> ParserResult<&str, IpAddr> {
        context(
            "ipv6_address",
            map_res(recognize(many1(ipv6_char)), |ipv6| ipv6.parse()),
        )
        .parse(input)
    }

    #[inline]
    fn ipv6_reference(input: &str) -> ParserResult<&str, IpAddr> {
        context(
            "ipv6_reference",
            delimited(tag("["), ipv6_address, tag("]")),
        )
        .parse(input)
    }

    pub(crate) fn host(input: &str) -> ParserResult<&str, Host> {
        context(
            "host",
            alt((
                map(hostname, Host::Name),
                map(ipv4_address, Host::Ip),
                map(ipv6_reference, Host::Ip),
            )),
        )
        .parse(input)
    }

    pub(crate) fn port(input: &str) -> ParserResult<&str, u16> {
        context(
            "port",
            map_res(recognize(many_m_n(1, 5, digit)), |digits| digits.parse()),
        )
        .parse(input)
    }

    pub(crate) fn hostport(input: &str) -> ParserResult<&str, (Host, Option<u16>)> {
        context("hostport", pair(host, opt(preceded(tag(":"), port)))).parse(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Host, HostnameString};
    use claims::{assert_err, assert_ok};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_valid_hostname_string() {
        let hostname_string = HostnameString::try_from("atlanta.com");
        assert_ok!(&hostname_string);
        if let Ok(hostname_string) = hostname_string {
            assert_eq!(hostname_string.as_str(), "atlanta.com");
            assert_eq!(format!("{}", hostname_string), "atlanta.com");
        }
    }

    #[test]
    fn test_invalid_hostname_string_invalid_character() {
        assert_err!(HostnameString::try_from("atl_anta.com"));
    }

    #[test]
    fn test_invalid_hostname_string_empty() {
        assert_err!(HostnameString::try_from(""));
    }

    #[test]
    fn test_invalid_hostname_string_dot_only() {
        assert_err!(HostnameString::try_from("."));
    }

    #[test]
    fn test_invalid_hostname_string_invalid_starting_dash() {
        assert_err!(HostnameString::try_from("-atlanta.com"));
    }

    #[test]
    fn test_invalid_hostname_string_invalid_trailing_dash() {
        assert_err!(HostnameString::try_from("atlanta-.com"));
    }

    #[test]
    fn test_valid_host_with_hostname() {
        let host = Host::try_from("atlanta.com");
        assert_ok!(&host);
        if let Ok(host) = host {
            assert_eq!(host, Host::Name(HostnameString::new("atlanta.com")));
            assert_eq!(format!("{}", host), "atlanta.com");
        }
    }

    #[test]
    fn test_valid_host_with_ipv4_address() {
        let host = Host::try_from("192.168.0.1");
        assert_ok!(&host);
        if let Ok(host) = host {
            assert_eq!(host, Host::Ip(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1))));
            assert_eq!(format!("{}", host), "192.168.0.1");
        }
    }

    fn test_valid_host_with_ipv6_address(
        input: &str,
        expected_ipv6_addr: Ipv6Addr,
        expected_display: &str,
    ) {
        let host = Host::try_from(input);
        assert_ok!(&host);
        if let Ok(host) = host {
            assert_eq!(host, Host::Ip(IpAddr::V6(expected_ipv6_addr)));
            assert_eq!(format!("{}", host), expected_display);
        }
    }

    #[test]
    fn test_valid_host_with_ipv6_address_01() {
        test_valid_host_with_ipv6_address(
            "[fe80::1]",
            Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0x0001),
            "[fe80::1]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_02() {
        test_valid_host_with_ipv6_address(
            "[2a01:e35:1387:1020:6233:4bff:fe0b:5663]",
            Ipv6Addr::new(
                0x2a01, 0x0e35, 0x1387, 0x1020, 0x6233, 0x4bff, 0xfe0b, 0x5663,
            ),
            "[2a01:e35:1387:1020:6233:4bff:fe0b:5663]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_03() {
        test_valid_host_with_ipv6_address(
            "[2a01:e35:1387:1020:6233::5663]",
            Ipv6Addr::new(0x2a01, 0x0e35, 0x1387, 0x1020, 0x6233, 0, 0, 0x5663),
            "[2a01:e35:1387:1020:6233::5663]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_04() {
        test_valid_host_with_ipv6_address(
            "[2001:DB8:0:0:8:800:200C:417A]",
            Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0x0008, 0x0800, 0x200c, 0x417a),
            "[2001:db8::8:800:200c:417a]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_05() {
        test_valid_host_with_ipv6_address(
            "[FF01:0:0:0:0:0:0:101]", // A multicast address
            Ipv6Addr::new(0xff01, 0, 0, 0, 0, 0, 0, 0x0101),
            "[ff01::101]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_06() {
        test_valid_host_with_ipv6_address(
            "[0:0:0:0:0:0:0:1]", // The loopback address
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x0001),
            "[::1]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_07() {
        test_valid_host_with_ipv6_address(
            "[0:0:0:0:0:0:0:0]", // The unspecified address
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0),
            "[::]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_08() {
        test_valid_host_with_ipv6_address(
            "[2001:DB8::8:800:200C:417A]",
            Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0x008, 0x0800, 0x200c, 0x417a),
            "[2001:db8::8:800:200c:417a]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_09() {
        test_valid_host_with_ipv6_address(
            "[FF01::101]",
            Ipv6Addr::new(0xff01, 0, 0, 0, 0, 0, 0, 0x0101),
            "[ff01::101]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_10() {
        test_valid_host_with_ipv6_address(
            "[::1]", // The loopback address
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x0001),
            "[::1]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_11() {
        test_valid_host_with_ipv6_address(
            "[::]", // The unspecified address
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0),
            "[::]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_12() {
        test_valid_host_with_ipv6_address(
            "[0:0:0:0:0:0:13.1.68.3]",
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0x0d01, 0x4403),
            "[::d01:4403]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_13() {
        test_valid_host_with_ipv6_address(
            "[0:0:0:0:0:FFFF:129.144.52.38]",
            Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x8190, 0x3426),
            "[::ffff:129.144.52.38]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_14() {
        test_valid_host_with_ipv6_address(
            "[::13.1.68.3]",
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0x0d01, 0x4403),
            "[::d01:4403]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_15() {
        test_valid_host_with_ipv6_address(
            "[::FFFF:129.144.52.38]",
            Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x8190, 0x3426),
            "[::ffff:129.144.52.38]",
        )
    }

    #[test]
    fn test_valid_host_with_ipv6_address_16() {
        test_valid_host_with_ipv6_address(
            "[2a01:e35:1387:1020::192.168.1.1]",
            Ipv6Addr::new(0x2a01, 0x0e35, 0x1387, 0x1020, 0, 0, 0xc0a8, 0x0101),
            "[2a01:e35:1387:1020::c0a8:101]",
        )
    }
}
