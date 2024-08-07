use std::convert::TryFrom;

use super::parser;
use crate::headers::generic_header::GenericHeader;
use crate::{
    AcceptEncodingHeader, AcceptHeader, AcceptLanguageHeader, AlertInfoHeader, AllowHeader,
    AuthenticationInfoHeader, AuthorizationHeader, CSeqHeader, CallIdHeader, CallInfoHeader,
    ContactHeader, ContentDispositionHeader, ContentEncodingHeader, ContentLanguageHeader,
    ContentLengthHeader, ContentTypeHeader, DateHeader, Error, ErrorInfoHeader, ExpiresHeader,
    FromHeader, InReplyToHeader, MaxForwardsHeader, MimeVersionHeader, MinExpiresHeader,
    OrganizationHeader, PriorityHeader, ProxyAuthenticateHeader, ProxyAuthorizationHeader,
    ProxyRequireHeader, RecordRouteHeader,
};

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

impl TryFrom<&str> for Header {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse(value)
    }
}

fn parse(input: &str) -> Result<Header, Error> {
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
