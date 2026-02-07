use itertools::join;
use std::hash::Hash;
use std::net::IpAddr;

use crate::common::value_collection::ValueCollection;
use crate::utils::compare_vectors;
use crate::{Host, Protocol, ViaParameter};

/// Representation of the list of vias from a `ViaHeader`.
///
/// This is usable as an iterator.
pub type Vias = ValueCollection<Via>;

/// Representation of a via contained in a `Via` header.
#[derive(Clone, Debug, Eq)]
pub struct Via {
    protocol: Protocol,
    host: Host,
    port: Option<u16>,
    parameters: Vec<ViaParameter>,
}

impl Via {
    pub(crate) fn new(
        protocol: Protocol,
        host: Host,
        port: Option<u16>,
        parameters: Vec<ViaParameter>,
    ) -> Self {
        Via {
            protocol,
            host,
            port,
            parameters,
        }
    }

    /// Get a reference to the protocol contained in the via.
    pub fn protocol(&self) -> &Protocol {
        &self.protocol
    }

    /// Get a reference to the host contained in the via.
    pub fn host(&self) -> &Host {
        &self.host
    }

    /// Get the port contained in the via.
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Get a reference to the parameters contained in the via.
    pub fn parameters(&self) -> &Vec<ViaParameter> {
        &self.parameters
    }

    /// Tell whether the Via contains a `ttl` parameter.
    pub fn has_ttl(&self) -> bool {
        self.parameters.iter().any(|p| p.is_ttl())
    }

    /// Get the value of the ttl parameter if there is one.
    pub fn ttl(&self) -> Option<u8> {
        self.parameters.iter().find_map(|p| p.ttl())
    }

    /// Tell whether the Via contains a `maddr` parameter.
    pub fn has_maddr(&self) -> bool {
        self.parameters.iter().any(|p| p.is_m_addr())
    }

    /// Get the value of the maddr parameter if there is one.
    pub fn maddr(&self) -> Option<Host> {
        self.parameters.iter().find_map(|p| p.maddr())
    }

    /// Tell whether the Via contains a `received` parameter.
    pub fn has_received(&self) -> bool {
        self.parameters.iter().any(|p| p.is_received())
    }

    /// Get the value of the received parameter if there is one.
    pub fn received(&self) -> Option<IpAddr> {
        self.parameters.iter().find_map(|p| p.received())
    }

    /// Tell whether the Via contains a `branch` parameter.
    pub fn has_branch(&self) -> bool {
        self.parameters.iter().any(|p| p.is_branch())
    }

    /// Get the value of the branch parameter if there is one.
    pub fn branch(&self) -> Option<String> {
        self.parameters.iter().find_map(|p| p.branch())
    }

    /// Tell whether the Via contains an `rport` parameter.
    pub fn has_rport(&self) -> bool {
        self.parameters.iter().any(|p| p.is_r_port())
    }

    /// Get the value of the rport parameter if there is one.
    pub fn rport(&self) -> Option<u16> {
        self.parameters.iter().find_map(|p| p.rport())
    }
}

impl std::fmt::Display for Via {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{}{}{}{}",
            self.protocol(),
            self.host(),
            if self.port().is_some() { ":" } else { "" },
            self.port().map(|p| p.to_string()).unwrap_or_default(),
            if self.parameters.is_empty() { "" } else { ";" },
            join(&self.parameters, ";")
        )
    }
}

impl PartialEq for Via {
    fn eq(&self, other: &Self) -> bool {
        self.protocol == other.protocol
            && self.host == other.host
            && self.port == other.port
            && compare_vectors(self.parameters(), other.parameters())
    }
}

impl Hash for Via {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.protocol.hash(state);
        self.host.hash(state);
        self.port.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

pub(crate) mod parser {
    use nom::{
        Parser,
        combinator::{map, opt},
        error::context,
        multi::many0,
        sequence::{pair, preceded},
    };

    use crate::{
        Host, Via,
        common::protocol::parser::sent_protocol,
        common::via_parameter::parser::via_params,
        parser::{ParserResult, colon, lws, semi},
        uris::host::parser::{host, port},
    };

    fn sent_by(input: &str) -> ParserResult<&str, (Host, Option<u16>)> {
        context("sent_by", pair(host, opt(preceded(colon, port)))).parse(input)
    }

    pub(crate) fn via_parm(input: &str) -> ParserResult<&str, Via> {
        context(
            "via_parm",
            map(
                (
                    sent_protocol,
                    lws,
                    sent_by,
                    many0(preceded(semi, via_params)),
                ),
                |(protocol, _, (host, port), params)| Via::new(protocol, host, port, params),
            ),
        )
        .parse(input)
    }
}
