//! TODO

pub mod accept_encoding_header;
pub mod accept_header;
pub mod accept_language_header;
pub mod alert_info_header;
pub mod allow_header;
pub mod authentication_info_header;
pub mod authorization_header;
pub mod call_id_header;
pub mod call_info_header;
pub mod contact_header;
pub mod content_disposition_header;
pub mod content_encoding_header;
pub mod content_language_header;
pub mod content_length_header;
pub mod content_type_header;
pub mod cseq_header;
pub mod date_header;
pub mod error_info_header;
pub mod expires_header;
pub mod from_header;
mod generic_header;
pub mod in_reply_to_header;
pub mod max_forwards_header;
pub mod mime_version_header;
pub mod min_expires_header;
pub mod organization_header;
pub mod priority_header;
pub mod proxy_authenticate_header;
pub mod proxy_authorization_header;
pub mod proxy_require_header;
pub mod record_route_header;

pub(crate) mod parser;

#[cfg(test)]
mod tests;

use accept_encoding_header::AcceptEncodingHeader;
use accept_header::AcceptHeader;
use accept_language_header::AcceptLanguageHeader;
use alert_info_header::AlertInfoHeader;
use allow_header::AllowHeader;
use authentication_info_header::AuthenticationInfoHeader;
use authorization_header::AuthorizationHeader;
use call_id_header::CallIdHeader;
use call_info_header::CallInfoHeader;
use contact_header::ContactHeader;
use content_disposition_header::ContentDispositionHeader;
use content_encoding_header::ContentEncodingHeader;
use content_language_header::ContentLanguageHeader;
use content_length_header::ContentLengthHeader;
use content_type_header::ContentTypeHeader;
use cseq_header::CSeqHeader;
use date_header::DateHeader;
use error_info_header::ErrorInfoHeader;
use expires_header::ExpiresHeader;
use from_header::FromHeader;
use in_reply_to_header::InReplyToHeader;
use max_forwards_header::MaxForwardsHeader;
use mime_version_header::MimeVersionHeader;
use min_expires_header::MinExpiresHeader;
use organization_header::OrganizationHeader;
use priority_header::PriorityHeader;
use proxy_authenticate_header::ProxyAuthenticateHeader;
use proxy_authorization_header::ProxyAuthorizationHeader;
use proxy_require_header::ProxyRequireHeader;
use record_route_header::RecordRouteHeader;

use crate::Error;
use generic_header::GenericHeader;

/// A trait defining the common accessors for all SIP message headers.
pub trait HeaderAccessor {
    /// Get the name of the header, as it has been parsed, whatever its case
    /// is.
    fn name(&self) -> &str;
    /// Get the separator of the header, as it has been parsed, containing
    /// any unnecessary space characters.
    fn separator(&self) -> &str;
    /// Get the value of the header, as it has been parsed, whatever its case
    /// is.
    fn value(&self) -> &str;

    /// Get the compact name of the header, if it has one.
    fn compact_name(&self) -> Option<&str>;
    /// Get the normalized name of the header, eg. `Call-ID`.
    fn normalized_name(&self) -> Option<&str>;
    /// Get the value of the header, cleaned from all unnecessary space
    /// characters and with case-insensitive parts converted to lowercase.
    fn normalized_value(&self) -> String;

    /// Tell whether the header has a compact name or not.
    fn has_compact_name(&self) -> bool {
        self.compact_name().is_some()
    }
    /// Get the compact format of the header.
    fn to_compact_string(&self) -> String {
        format!(
            "{}: {}",
            self.compact_name()
                .or_else(|| self.normalized_name())
                .or_else(|| self.name().into())
                .unwrap(),
            self.normalized_value()
        )
    }
    /// Get the normalized format of the header.
    fn to_normalized_string(&self) -> String {
        format!(
            "{}: {}",
            self.normalized_name()
                .or_else(|| self.name().into())
                .unwrap(),
            self.normalized_value()
        )
    }
}

macro_rules! generic_header_accessors {
    (
        $header:ident
    ) => {
        fn name(&self) -> &str {
            self.$header.name()
        }
        fn separator(&self) -> &str {
            self.$header.separator()
        }
        fn value(&self) -> &str {
            self.$header.value()
        }
    };
}
pub(crate) use generic_header_accessors;

macro_rules! headers {
    (
        $(
            $(#[$docs:meta])*
            ($variant:ident, $type:ident),
        )+
    ) => {
        /// Representation of a SIP message header.
        #[derive(Clone, Debug)]
        pub enum Header {
            $(
                $(#[$docs])*
                $variant($type),
            )+
        }

        impl std::fmt::Display for Header {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        $(
                            Header::$variant(header) => header.to_string(),
                        )+
                    }
                )
            }
        }
    }
}

headers! {
    /// An Accept message header.
    (Accept, AcceptHeader),
    /// An Accept-Encoding message header.
    (AcceptEncoding, AcceptEncodingHeader),
    /// An Accept-Language message header.
    (AcceptLanguage, AcceptLanguageHeader),
    /// An Alert-Info message header.
    (AlertInfo, AlertInfoHeader),
    /// An Allow message header.
    (Allow, AllowHeader),
    /// An Authentication-Info header.
    (AuthenticationInfo, AuthenticationInfoHeader),
    /// An Authorization header.
    (Authorization, AuthorizationHeader),
    /// A Call-ID header.
    (CallId, CallIdHeader),
    /// A Call-Info header.
    (CallInfo, CallInfoHeader),
    /// A Contact header.
    (Contact, ContactHeader),
    /// A Content-Disposition header.
    (ContentDisposition, ContentDispositionHeader),
    /// A Content-Encoding header.
    (ContentEncoding, ContentEncodingHeader),
    /// A Content-Language header.
    (ContentLanguage, ContentLanguageHeader),
    /// A Content-Length header.
    (ContentLength, ContentLengthHeader),
    /// A Content-Type header.
    (ContentType, ContentTypeHeader),
    /// A CSeq header.
    (CSeq, CSeqHeader),
    /// A Date header.
    (Date, DateHeader),
    /// An Error-Info header.
    (ErrorInfo, ErrorInfoHeader),
    /// An Expires header.
    (Expires, ExpiresHeader),
    /// A From header.
    (From, FromHeader),
    /// An In-Reply-To header.
    (InReplyTo, InReplyToHeader),
    /// A Max-Forwards header.
    (MaxForwards, MaxForwardsHeader),
    /// A MIME-Version header.
    (MimeVersion, MimeVersionHeader),
    /// A Min-Expires header.
    (MinExpires, MinExpiresHeader),
    /// An Organization header.
    (Organization, OrganizationHeader),
    /// A Priority header.
    (Priority, PriorityHeader),
    /// A Proxy-Authenticate header.
    (ProxyAuthenticate, ProxyAuthenticateHeader),
    /// A Proxy-Authorization header.
    (ProxyAuthorization, ProxyAuthorizationHeader),
    /// A Proxy-Require header.
    (ProxyRequire, ProxyRequireHeader),
    /// A Record-Route header.
    (RecordRoute, RecordRouteHeader),
    /// An extension header.
    (ExtensionHeader, GenericHeader),
}

impl Header {
    /// Try to create a `Header` from a slice of bytes.
    #[inline]
    pub fn from_bytes(input: &[u8]) -> Result<Header, Error> {
        parse(input)
    }
}

impl TryFrom<&str> for Header {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Header::from_bytes(value.as_bytes())
    }
}

fn parse(input: &[u8]) -> Result<Header, Error> {
    match parser::message_header(input) {
        Ok((rest, uri)) => {
            if !rest.is_empty() {
                Err(Error::RemainingUnparsedData)
            } else {
                Ok(uri)
            }
        }
        Err(e) => Err(Error::InvalidMessageHeader(e.to_string())),
    }
}
