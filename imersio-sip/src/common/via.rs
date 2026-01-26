use itertools::join;
use std::hash::Hash;

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
        combinator::{map, opt},
        error::context,
        multi::many0,
        sequence::{pair, preceded},
        Parser,
    };

    use crate::{
        common::protocol::parser::sent_protocol,
        common::via_parameter::parser::via_params,
        parser::{colon, lws, semi, ParserResult},
        uris::host::parser::{host, port},
        Host, Via,
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
