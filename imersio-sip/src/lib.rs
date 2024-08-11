//! A library providing common SIP types and allowing generation and parsing of SIP messages.
//!
//! TODO

#![deny(warnings, missing_docs, missing_debug_implementations)]

pub mod headers;
pub mod request;
pub mod response;
pub mod uri;

mod common;
mod error;
mod parser;
mod utils;

pub use bytes::Bytes;

pub use crate::common::{
    accept_encoding::{AcceptEncoding, AcceptEncodings},
    accept_language::{AcceptLanguage, AcceptLanguages},
    accept_parameter::AcceptParameter,
    accept_range::{AcceptRange, AcceptRanges},
    alert::{Alert, Alerts},
    algorithm::Algorithm,
    auth_parameter::{AuthParameter, AuthParameters},
    authentication_info::{AuthenticationInfo, AuthenticationInfos},
    call_id::{CallId, CallIds},
    call_info::{CallInfo, CallInfos},
    call_info_parameter::CallInfoParameter,
    challenge::Challenge,
    contact::{Contact, Contacts},
    contact_parameter::ContactParameter,
    content_encoding::{ContentEncoding, ContentEncodings},
    content_language::{ContentLanguage, ContentLanguages},
    credentials::Credentials,
    disposition_parameter::DispositionParameter,
    disposition_type::DispositionType,
    domain_uri::{DomainUri, DomainUris},
    error_uri::{ErrorUri, ErrorUris},
    from_parameter::{FromParameter, FromParameters},
    generic_parameter::{GenericParameter, GenericParameters},
    handling::Handling,
    media_parameter::MediaParameter,
    media_range::MediaRange,
    media_type::MediaType,
    message_qop::{MessageQop, MessageQops},
    method::{Method, Methods},
    name_address::NameAddress,
    option_tag::{OptionTag, OptionTags},
    priority::Priority,
    product::Product,
    reason::Reason,
    retry_parameter::RetryParameter,
    route::{Route, Routes},
    server_value::{ServerValue, ServerValues},
    stale::Stale,
    status_code::StatusCode,
    to_parameter::{ToParameter, ToParameters},
    version::Version,
};
pub use crate::error::Error;
pub use crate::headers::{
    accept_encoding_header::AcceptEncodingHeader, accept_header::AcceptHeader,
    accept_language_header::AcceptLanguageHeader, alert_info_header::AlertInfoHeader,
    allow_header::AllowHeader, authentication_info_header::AuthenticationInfoHeader,
    authorization_header::AuthorizationHeader, call_id_header::CallIdHeader,
    call_info_header::CallInfoHeader, contact_header::ContactHeader,
    content_disposition_header::ContentDispositionHeader,
    content_encoding_header::ContentEncodingHeader, content_language_header::ContentLanguageHeader,
    content_length_header::ContentLengthHeader, content_type_header::ContentTypeHeader,
    cseq_header::CSeqHeader, date_header::DateHeader, error_info_header::ErrorInfoHeader,
    expires_header::ExpiresHeader, from_header::FromHeader, in_reply_to_header::InReplyToHeader,
    max_forwards_header::MaxForwardsHeader, mime_version_header::MimeVersionHeader,
    min_expires_header::MinExpiresHeader, organization_header::OrganizationHeader,
    priority_header::PriorityHeader, proxy_authenticate_header::ProxyAuthenticateHeader,
    proxy_authorization_header::ProxyAuthorizationHeader, proxy_require_header::ProxyRequireHeader,
    record_route_header::RecordRouteHeader, reply_to_header::ReplyToHeader,
    require_header::RequireHeader, retry_after_header::RetryAfterHeader, route_header::RouteHeader,
    server_header::ServerHeader, subject_header::SubjectHeader, supported_header::SupportedHeader,
    timestamp_header::TimestampHeader, to_header::ToHeader, Header,
};
pub use crate::request::Request;
pub use crate::response::Response;
pub use crate::uri::{
    AbsoluteUri, HostPort, SipUri, Uri, UriHeaders, UriParameters, UriScheme, UserInfo,
};
