//! TODO

mod accept_encoding_header;
mod accept_header;
mod accept_language_header;
mod alert_info_header;
mod allow_header;
mod authentication_info_header;
mod authorization_header;
mod call_id_header;
mod call_info_header;
mod contact_header;
mod content_disposition_header;
mod content_encoding_header;
mod content_language_header;
mod content_length_header;
mod content_type_header;
mod cseq_header;
mod date_header;
mod error_info_header;
mod expires_header;
mod from_header;
mod generic_header;
mod in_reply_to_header;
mod max_forwards_header;
mod mime_version_header;
mod min_expires_header;
mod organization_header;
mod priority_header;
mod proxy_authenticate_header;

pub(crate) mod parser;

#[cfg(test)]
mod tests;

use std::str::FromStr;

pub use accept_encoding_header::{AcceptEncoding, AcceptEncodingHeader, AcceptEncodings};
pub use accept_header::{AcceptHeader, AcceptRange, AcceptRanges};
pub use accept_language_header::{AcceptLanguageHeader, Language, Languages};
pub use alert_info_header::{Alert, AlertInfoHeader, Alerts};
pub use allow_header::{AllowHeader, Methods};
pub use authentication_info_header::{AInfo, AInfos, AuthenticationInfoHeader};
pub use authorization_header::AuthorizationHeader;
pub use call_id_header::CallIdHeader;
pub use call_info_header::{CallInfo, CallInfoHeader, CallInfoParameter, CallInfos};
pub use contact_header::{Contact, ContactHeader, ContactParameter, Contacts};
pub use content_disposition_header::{
    ContentDispositionHeader, DispositionParameter, DispositionType, HandlingValue,
};
pub use content_encoding_header::{ContentEncodingHeader, ContentEncodings};
pub use content_language_header::{ContentLanguage, ContentLanguageHeader, ContentLanguages};
pub use content_length_header::ContentLengthHeader;
pub use content_type_header::{ContentTypeHeader, MediaParameter, MediaType};
pub use cseq_header::CSeqHeader;
pub use date_header::DateHeader;
pub use error_info_header::{ErrorInfoHeader, ErrorUri, ErrorUris};
pub use expires_header::ExpiresHeader;
pub use from_header::{FromHeader, FromParameter, FromParameters};
use generic_header::GenericHeader;
pub use in_reply_to_header::{CallIds, InReplyToHeader};
pub use max_forwards_header::MaxForwardsHeader;
pub use mime_version_header::MimeVersionHeader;
pub use min_expires_header::MinExpiresHeader;
pub use organization_header::OrganizationHeader;
pub use priority_header::{PriorityHeader, PriorityValue};
pub use proxy_authenticate_header::ProxyAuthenticateHeader;

use crate::Error;

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

impl FromStr for Header {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Header::from_bytes(s.as_bytes())
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
