use std::cmp::Ordering;
use std::hash::Hash;

use crate::{SipError, TokenString};

/// Representation of a transport contained in a Via header or in a transport uri parameter.
#[derive(Clone, Debug, Eq, derive_more::IsVariant)]
pub enum Transport {
    /// UDP transport.
    Udp,
    /// TCP transport.
    Tcp,
    /// TLS transport.
    Tls,
    /// SCTP transport
    Sctp,
    /// Any other transport.
    Other(TokenString),
}

impl Transport {
    pub(crate) fn new(transport: TokenString) -> Self {
        match transport.to_ascii_lowercase().as_str() {
            "udp" => Self::Udp,
            "tcp" => Self::Tcp,
            "tls" => Self::Tls,
            "sctp" => Self::Sctp,
            _ => Self::Other(transport),
        }
    }

    /// Get the value of the transport.
    pub fn value(&self) -> &str {
        match self {
            Self::Udp => "UDP",
            Self::Tcp => "TCP",
            Self::Tls => "TLS",
            Self::Sctp => "SCTP",
            Self::Other(value) => value,
        }
    }
}

impl std::fmt::Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq<Transport> for Transport {
    fn eq(&self, other: &Transport) -> bool {
        match (self, other) {
            (Self::Udp, Self::Udp)
            | (Self::Tcp, Self::Tcp)
            | (Self::Tls, Self::Tls)
            | (Self::Sctp, Self::Sctp) => true,
            (Self::Other(a), Self::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialOrd for Transport {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transport {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(other.value())
    }
}

impl Hash for Transport {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().to_ascii_lowercase().hash(state);
    }
}

impl TryFrom<&str> for Transport {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Transport::new(TokenString::try_from(value)?))
    }
}

pub(crate) mod parser {
    use nom::{branch::alt, bytes::complete::tag_no_case, combinator::map, error::context, Parser};

    use crate::{
        parser::{token, ParserResult},
        TokenString, Transport,
    };

    pub(crate) fn transport(input: &str) -> ParserResult<&str, Transport> {
        context(
            "transport",
            map(
                alt((
                    map(
                        alt((
                            tag_no_case("UDP"),
                            tag_no_case("TCP"),
                            tag_no_case("TLS"),
                            tag_no_case("SCTP"),
                        )),
                        TokenString::new,
                    ),
                    other_transport,
                )),
                Transport::new,
            ),
        )
        .parse(input)
    }

    #[inline]
    fn other_transport(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }
}
