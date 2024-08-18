use crate::{TokenString, Transport};
use std::hash::Hash;

/// Representation of a protocol, containing its name and version.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Protocol {
    name: TokenString,
    version: TokenString,
    transport: Transport,
}

impl Protocol {
    pub(crate) fn new(name: TokenString, version: TokenString, transport: Transport) -> Self {
        Protocol {
            name,
            version,
            transport,
        }
    }

    /// Get the name of the protocol.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the protocol.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get a reference to the transport of the protocol.
    pub fn transport(&self) -> &Transport {
        &self.transport
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.name().to_ascii_uppercase(),
            self.version(),
            self.transport()
        )
    }
}

pub(crate) mod parser {
    use crate::common::transport::parser::transport;
    use crate::parser::{slash, token, ParserResult};
    use crate::{Protocol, TokenString};
    use nom::{
        branch::alt, bytes::complete::tag_no_case, combinator::map, error::context, sequence::tuple,
    };

    pub(crate) fn sent_protocol(input: &str) -> ParserResult<&str, Protocol> {
        context(
            "sent_protocol",
            map(
                tuple((protocol_name, slash, protocol_version, slash, transport)),
                |(name, _, version, _, transport)| Protocol::new(name, version, transport),
            ),
        )(input)
    }

    #[inline]
    fn protocol_name(input: &str) -> ParserResult<&str, TokenString> {
        alt((
            map(tag_no_case("SIP"), |p: &str| {
                TokenString::new(p.to_ascii_uppercase())
            }),
            token,
        ))(input)
    }

    #[inline]
    fn protocol_version(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }
}
