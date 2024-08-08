use nom::error::convert_error;
use std::convert::TryFrom;

use crate::headers::generic_header::GenericHeader;
use crate::{
    AcceptEncodingHeader, AcceptHeader, AcceptLanguageHeader, AlertInfoHeader, AllowHeader,
    AuthenticationInfoHeader, AuthorizationHeader, CSeqHeader, CallIdHeader, CallInfoHeader,
    ContactHeader, ContentDispositionHeader, ContentEncodingHeader, ContentLanguageHeader,
    ContentLengthHeader, ContentTypeHeader, DateHeader, Error, ErrorInfoHeader, ExpiresHeader,
    FromHeader, InReplyToHeader, MaxForwardsHeader, MimeVersionHeader, MinExpiresHeader,
    OrganizationHeader, PriorityHeader, ProxyAuthenticateHeader, ProxyAuthorizationHeader,
    ProxyRequireHeader, RecordRouteHeader, ReplyToHeader, RequireHeader,
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
    /// A Reply-To header.
    (ReplyTo, ReplyToHeader),
    /// A Require header.
    (Require, RequireHeader),
    /// An extension header.
    (ExtensionHeader, GenericHeader),
}

impl TryFrom<&str> for Header {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::message_header(value) {
            Ok((rest, uri)) => {
                if !rest.is_empty() {
                    Err(Error::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(uri)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(Error::InvalidMessageHeader(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidMessageHeader(format!(
                "Incomplete message header `{}`",
                value
            ))),
        }
    }
}

mod parser {
    use crate::headers::{
        accept_encoding_header::parser::accept_encoding, accept_header::parser::accept,
        accept_language_header::parser::accept_language, alert_info_header::parser::alert_info,
        allow_header::parser::allow, authentication_info_header::parser::authentication_info,
        authorization_header::parser::authorization, call_id_header::parser::call_id,
        call_info_header::parser::call_info, contact_header::parser::contact,
        content_disposition_header::parser::content_disposition,
        content_encoding_header::parser::content_encoding,
        content_language_header::parser::content_language,
        content_length_header::parser::content_length, content_type_header::parser::content_type,
        cseq_header::parser::cseq, date_header::parser::date,
        error_info_header::parser::error_info, expires_header::parser::expires,
        from_header::parser::from, generic_header::parser::extension_header,
        in_reply_to_header::parser::in_reply_to, max_forwards_header::parser::max_forwards,
        mime_version_header::parser::mime_version, min_expires_header::parser::min_expires,
        organization_header::parser::organization, priority_header::parser::priority,
        proxy_authenticate_header::parser::proxy_authenticate,
        proxy_authorization_header::parser::proxy_authorization,
        proxy_require_header::parser::proxy_require, record_route_header::parser::record_route,
        reply_to_header::parser::reply_to, require_header::parser::require,
    };
    use crate::{parser::ParserResult, Header};
    use nom::{branch::alt, error::context};

    pub(super) fn message_header(input: &str) -> ParserResult<&str, Header> {
        context(
            "message_header",
            alt((
                alt((
                    accept,
                    accept_encoding,
                    accept_language,
                    alert_info,
                    allow,
                    authentication_info,
                    authorization,
                    call_id,
                    call_info,
                    contact,
                    content_disposition,
                    content_encoding,
                    content_language,
                    content_length,
                    content_type,
                    cseq,
                    date,
                    error_info,
                    expires,
                    from,
                    in_reply_to,
                )),
                alt((
                    max_forwards,
                    mime_version,
                    min_expires,
                    organization,
                    priority,
                    proxy_authenticate,
                    proxy_authorization,
                    proxy_require,
                    record_route,
                    reply_to,
                    require,
                    extension_header,
                )),
            )),
        )(input)
    }
}
