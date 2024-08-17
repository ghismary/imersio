use crate::Transport;
use std::hash::Hash;

/// Representation of a protocol, containing its name and version.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Protocol {
    name: String,
    version: String,
    transport: Transport,
}

impl Protocol {
    pub(crate) fn new<S: Into<String>>(name: S, version: S, transport: Transport) -> Self {
        Protocol {
            name: name.into(),
            version: version.into(),
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
    use crate::Protocol;
    use nom::{
        branch::alt, bytes::complete::tag_no_case, combinator::map, error::context, sequence::tuple,
    };

    pub(crate) fn sent_protocol(input: &str) -> ParserResult<&str, Protocol> {
        context(
            "sent_protocol",
            map(
                tuple((protocol_name, slash, protocol_version, slash, transport)),
                |(name, _, version, _, transport)| {
                    Protocol::new(name.to_ascii_uppercase(), version.to_string(), transport)
                },
            ),
        )(input)
    }

    fn protocol_name(input: &str) -> ParserResult<&str, &str> {
        alt((tag_no_case("SIP"), token))(input)
    }

    #[inline]
    fn protocol_version(input: &str) -> ParserResult<&str, &str> {
        token(input)
    }
}
