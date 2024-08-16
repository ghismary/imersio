use std::error::Error;

use derive_more::Display;

/// A generic error for SIP
#[derive(Debug, Display, PartialEq)]
pub enum SipError {
    /// Failed converting AInfo to AuthParam.
    #[display("Failed converting AInfo to AuthParam")]
    FailedConvertingAInfoToAuthParam,
    /// Invalid call id.
    #[display("Invalid call id: `{_0}`")]
    InvalidCallId(String),
    /// Invalid content encoding.
    #[display("Invalid content encoding: `{_0}`")]
    InvalidContentEncoding(String),
    /// Invalid content language.
    #[display("Invalid content language: `{_0}`")]
    InvalidContentLanguage(String),
    /// Invalid message.
    #[display("Invalid message:\n{_0}")]
    InvalidMessage(String),
    /// Invalid message header.
    #[display("Invalid message header: `{_0}`")]
    InvalidMessageHeader(String),
    /// Invalid method.
    #[display("Invalid method: `{_0}`")]
    InvalidMethod(String),
    /// Invalid option tag.
    #[display("Invalid option tag: `{_0}`")]
    InvalidOptionTag(String),
    /// Invalid response reason.
    #[display("Invalid reason: `{_0}`")]
    InvalidReason(String),
    /// Invalid request.
    #[display("Invalid request:\n{_0}")]
    InvalidRequest(String),
    /// Invalid response.
    #[display("Invalid response:\n{_0}")]
    InvalidResponse(String),
    /// Invalid response status code.
    #[display("Invalid status code: `{_0}`")]
    InvalidStatusCode(String),
    /// Invalid URI.
    #[display("Invalid uri: `{_0}`")]
    InvalidUri(String),
    /// Invalid SIP version.
    #[display("Invalid sip version: `{_0}`")]
    InvalidVersion(String),
    /// Invalid warning code.
    #[display("Invalid warning code: `{_0}`")]
    InvalidWarnCode(String),
    /// Invalid warning agent.
    #[display("Invalid warning agent: `{_0}`")]
    InvalidWarnAgent(String),
    /// Remaining unparsed data.
    #[display("Remaining unparsed data")]
    RemainingUnparsedData(String),
}

impl Error for SipError {}
