use std::cmp::Ordering;
use std::net::IpAddr;

use crate::common::generic_parameter::generic_parameter_display;
use crate::uris::host::parser::host;
use crate::{GenericParameter, Host, TokenString};

/// Representation of a via parameter.
#[derive(Clone, Debug, Eq, Hash, PartialEq, derive_more::IsVariant)]
pub enum ViaParameter {
    /// A `ttl` parameter.
    Ttl(String),
    /// A `maddr` parameter.
    MAddr(String),
    /// A `received` parameter.
    Received(String),
    /// A `branch` parameter.
    Branch(String),
    /// A `rport` parameter.
    RPort(Option<String>),
    /// Any other parameter.
    Other(GenericParameter<TokenString>),
}

impl ViaParameter {
    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Ttl(_) => "ttl",
            Self::MAddr(_) => "maddr",
            Self::Received(_) => "received",
            Self::Branch(_) => "branch",
            Self::RPort(_) => "rport",
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Ttl(value) => Some(value),
            Self::MAddr(value) => Some(value),
            Self::Received(value) => Some(value),
            Self::Branch(value) => Some(value),
            Self::RPort(value) => value.as_ref().map(|value| value.as_str()),
            Self::Other(value) => value.value(),
        }
    }

    /// Get the ttl value of the parameter if this is a `ttl` parameter.
    pub fn ttl(&self) -> Option<u8> {
        match self {
            Self::Ttl(value) => value.parse().ok(),
            _ => None,
        }
    }

    /// Get the maddr value of the parameter if this is an `maddr` parameter.
    pub fn maddr(&self) -> Option<Host> {
        match self {
            Self::MAddr(value) => host(value).ok().map(|(_, host)| host),
            _ => None,
        }
    }

    /// Get the received value of the parameter if this is a `received` parameter.
    pub fn received(&self) -> Option<IpAddr> {
        match self {
            Self::Received(value) => value.parse().ok(),
            _ => None,
        }
    }

    /// Get the branch value of the parameter if this is a `branch` parameter.
    pub fn branch(&self) -> Option<String> {
        match self {
            Self::Branch(value) => Some(value.clone()),
            _ => None,
        }
    }

    /// Get the rport value of the parameter if this is an `rport` parameter.
    pub fn rport(&self) -> Option<u16> {
        match self {
            Self::RPort(value) => value.as_ref().and_then(|value| value.parse().ok()),
            _ => None,
        }
    }
}

impl std::fmt::Display for ViaParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        generic_parameter_display(self.key(), self.value(), f)
    }
}

impl PartialOrd for ViaParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ViaParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => self.value().cmp(&other.value()),
            ord => ord,
        }
    }
}

impl From<GenericParameter<TokenString>> for ViaParameter {
    fn from(value: GenericParameter<TokenString>) -> Self {
        Self::Other(value)
    }
}

pub(crate) mod parser {
    use nom::{
        Parser,
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{consumed, map, opt, recognize, verify},
        error::context,
        multi::many_m_n,
        sequence::{pair, preceded, separated_pair},
    };

    use crate::{
        ViaParameter,
        common::generic_parameter::parser::generic_param,
        parser::{ParserResult, digit, equal, token},
        uris::host::parser::{host, ipv4_address, ipv6_address, port},
    };

    pub(crate) fn via_params(input: &str) -> ParserResult<&str, ViaParameter> {
        context(
            "via_params",
            alt((
                via_ttl,
                via_maddr,
                via_received,
                via_branch,
                response_port,
                via_extension,
            )),
        )
        .parse(input)
    }

    fn via_ttl(input: &str) -> ParserResult<&str, ViaParameter> {
        context(
            "via_ttl",
            map(
                separated_pair(
                    tag_no_case("ttl"),
                    equal,
                    recognize(verify(ttl, is_valid_ttl)),
                ),
                |(_, ttl)| ViaParameter::Ttl(ttl.to_string()),
            ),
        )
        .parse(input)
    }

    fn via_maddr(input: &str) -> ParserResult<&str, ViaParameter> {
        context(
            "via_maddr",
            map(
                separated_pair(tag_no_case("maddr"), equal, consumed(host)),
                |(_, (host, _))| ViaParameter::MAddr(host.to_string()),
            ),
        )
        .parse(input)
    }

    fn via_received(input: &str) -> ParserResult<&str, ViaParameter> {
        context(
            "via_received",
            map(
                separated_pair(
                    tag_no_case("received"),
                    equal,
                    consumed(alt((ipv4_address, ipv6_address))),
                ),
                |(_, (ip, _))| ViaParameter::Received(ip.to_string()),
            ),
        )
        .parse(input)
    }

    fn via_branch(input: &str) -> ParserResult<&str, ViaParameter> {
        context(
            "via_branch",
            map(
                separated_pair(tag_no_case("branch"), equal, token),
                |(_, branch)| ViaParameter::Branch(branch.to_string()),
            ),
        )
        .parse(input)
    }

    fn response_port(input: &str) -> ParserResult<&str, ViaParameter> {
        context(
            "response_port",
            map(
                pair(tag_no_case("rport"), opt(preceded(equal, recognize(port)))),
                |(_, port)| ViaParameter::RPort(port.map(|port| port.to_string())),
            ),
        )
        .parse(input)
    }

    fn via_extension(input: &str) -> ParserResult<&str, ViaParameter> {
        context("via_extension", map(generic_param, Into::into)).parse(input)
    }

    fn is_valid_ttl(value: &str) -> bool {
        value.parse::<u8>().is_ok()
    }
    fn ttl(input: &str) -> ParserResult<&str, &str> {
        recognize(many_m_n(1, 3, digit)).parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::wrapped_string::WrappedString;
    use std::net::Ipv4Addr;

    #[test]
    fn test_via_ttl_parameter() {
        let parameter = ViaParameter::Ttl("16".to_string());
        assert_eq!(parameter.key(), "ttl");
        assert_eq!(parameter.value(), Some("16"));
        assert_eq!(parameter.ttl(), Some(16));
    }

    #[test]
    fn test_via_maddr_parameter() {
        let parameter = ViaParameter::MAddr("192.0.2.1".to_string());
        assert_eq!(parameter.key(), "maddr");
        assert_eq!(parameter.value(), Some("192.0.2.1"));
        assert_eq!(
            parameter.maddr(),
            Some(Host::Ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))))
        );
    }

    #[test]
    fn test_via_received_parameter() {
        let parameter = ViaParameter::Received("192.0.2.207".to_string());
        assert_eq!(parameter.key(), "received");
        assert_eq!(parameter.value(), Some("192.0.2.207"));
        assert_eq!(
            parameter.received(),
            Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 207)))
        );
    }

    #[test]
    fn test_via_branch_parameter() {
        let parameter = ViaParameter::Branch("z9hG4bK77asjd".to_string());
        assert_eq!(parameter.key(), "branch");
        assert_eq!(parameter.value(), Some("z9hG4bK77asjd"));
        assert_eq!(parameter.branch(), Some("z9hG4bK77asjd".to_string()));
    }

    #[test]
    fn test_via_rport_parameter() {
        let parameter = ViaParameter::RPort(Some("5060".to_string()));
        assert_eq!(parameter.key(), "rport");
        assert_eq!(parameter.value(), Some("5060"));
        assert_eq!(parameter.rport(), Some(5060));
    }

    #[test]
    fn test_via_rport_parameter_without_value() {
        let parameter = ViaParameter::RPort(None);
        assert_eq!(parameter.key(), "rport");
        assert_eq!(parameter.value(), None);
        assert_eq!(parameter.rport(), None);
    }

    #[test]
    fn test_via_other_parameter() {
        let parameter = ViaParameter::Other(GenericParameter::new(
            TokenString::new("other"),
            Some(WrappedString::new_not_wrapped(TokenString::new("value"))),
        ));
        assert_eq!(parameter.key(), "other");
        assert_eq!(parameter.value(), Some("value"));
    }

    #[test]
    fn test_via_parameter_from_generic_parameter() {
        let parameter = ViaParameter::from(GenericParameter::new(
            TokenString::new("other"),
            Some(WrappedString::new_not_wrapped(TokenString::new("value"))),
        ));
        assert_eq!(parameter.key(), "other");
        assert_eq!(parameter.value(), Some("value"));
    }

    #[test]
    fn test_via_parameter_cmp() {
        let (remaining, first_parameter) = parser::via_params("other=value").unwrap();
        assert_eq!(remaining, "");
        let second_parameter = ViaParameter::from(GenericParameter::new(
            TokenString::new("other"),
            Some(WrappedString::new_not_wrapped(TokenString::new("value"))),
        ));
        assert_eq!(first_parameter, second_parameter);
    }
}
