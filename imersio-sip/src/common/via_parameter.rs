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
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
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
        combinator::{consumed, map, recognize, verify},
        error::context,
        multi::many_m_n,
        sequence::separated_pair,
    };

    use crate::{
        ViaParameter,
        common::generic_parameter::parser::generic_param,
        parser::{ParserResult, digit, equal, token},
        uris::host::parser::{host, ipv4_address, ipv6_address},
    };

    pub(crate) fn via_params(input: &str) -> ParserResult<&str, ViaParameter> {
        context(
            "via_params",
            alt((via_ttl, via_maddr, via_received, via_branch, via_extension)),
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
