use thiserror::Error;

/// A generic error for SIP
#[derive(Error, Debug)]
pub enum Error {
    /// Remaining unparsed data.
    #[error("remaining unparsed data")]
    RemainingUnparsedData,
    /// Invalid message header.
    #[error("invalid message header")]
    InvalidMessageHeader(String),
    /// Invalid method.
    #[error("invalid method")]
    InvalidMethod(String),
    /// Invalid response reason.
    #[error("invalid reason")]
    InvalidReason(String),
    /// Invalid request.
    #[error("invalid request")]
    InvalidRequest(String),
    /// Invalid response.
    #[error("invalid response")]
    InvalidResponse(String),
    /// Invalid response status code.
    #[error("invalid status code")]
    InvalidStatusCode(String),
    /// Invalid URI.
    #[error("invalid uri")]
    InvalidUri(String),
    /// Invalid SIP version.
    #[error("invalid sip version")]
    InvalidVersion(String),
}

impl From<std::convert::Infallible> for Error {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}
