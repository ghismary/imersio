use thiserror::Error;

/// A generic error for SIP
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Failed converting AInfo to AuthParam.
    #[error("failed converting AInfo to AuthParam")]
    FailedConvertingAInfoToAuthParam,
    /// Invalid call id.
    #[error("invalid call id")]
    InvalidCallId(String),
    /// Invalid content encoding.
    #[error("invalid content encoding")]
    InvalidContentEncoding(String),
    /// Invalid content language.
    #[error("invalid content language")]
    InvalidContentLanguage(String),
    /// Invalid message header.
    #[error("invalid message header")]
    InvalidMessageHeader(String),
    /// Invalid method.
    #[error("invalid method")]
    InvalidMethod(String),
    /// Invalid option tag.
    #[error("invalid option tag")]
    InvalidOptionTag(String),
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
    /// Remaining unparsed data.
    #[error("remaining unparsed data")]
    RemainingUnparsedData(String),
}

impl From<std::convert::Infallible> for Error {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}
