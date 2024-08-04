use chrono::{DateTime, Utc};
use std::borrow::Cow;

use nom::error::{ErrorKind, ParseError, VerboseError};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::{consumed, cut, map, opt, recognize},
    error::context,
    multi::{count, many0, many1, many_m_n, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};

use crate::common::accept_encoding::AcceptEncoding;
use crate::common::accept_language::AcceptLanguage;
use crate::common::accept_range::AcceptRange;
use crate::common::alert::Alert;
use crate::common::auth_parameter::{AuthParameter, AuthParameters};
use crate::common::authentication_info::AuthenticationInfo;
use crate::common::call_info::CallInfo;
use crate::common::call_info_parameter::CallInfoParameter;
use crate::common::challenge::Challenge;
use crate::common::contact::{Contact, Contacts};
use crate::common::contact_parameter::ContactParameter;
use crate::common::content_language::ContentLanguage;
use crate::common::credentials::Credentials;
use crate::common::disposition_parameter::DispositionParameter;
use crate::common::disposition_type::DispositionType;
use crate::common::domain_uri::DomainUri;
use crate::common::error_uri::ErrorUri;
use crate::common::from_parameter::FromParameter;
use crate::common::handling::Handling;
use crate::common::media_parameter::MediaParameter;
use crate::common::media_type::MediaType;
use crate::common::priority::Priority;
use crate::common::stale::Stale;
use crate::header::date_header::DateHeader;
use crate::header::from_header::FromHeader;
use crate::header::max_forwards_header::MaxForwardsHeader;
use crate::header::mime_version_header::MimeVersionHeader;
use crate::parser::sp;
use crate::{
    common::{
        accept_parameter::AcceptParameter, algorithm::Algorithm, content_encoding::ContentEncoding,
        media_range::MediaRange, message_qop::MessageQop, name_address::NameAddress,
        option_tag::OptionTag, wrapped_string::WrappedString,
    },
    method::parser::method,
    parser::{
        alpha, comma, digit, equal, hcolon, laquot, ldquot, lhex, lws, param, pchar, quoted_string,
        raquot, rdquot, semi, slash, star, text_utf8_trim, text_utf8char, token, utf8_cont, word,
        ParserResult,
    },
    uri::parser::{absolute_uri, host, request_uri, sip_uri},
    GenericParameter, Uri,
};

use super::{
    accept_encoding_header::AcceptEncodingHeader, accept_header::AcceptHeader,
    accept_language_header::AcceptLanguageHeader, alert_info_header::AlertInfoHeader,
    allow_header::AllowHeader, authentication_info_header::AuthenticationInfoHeader,
    authorization_header::AuthorizationHeader, call_id_header::CallIdHeader,
    call_info_header::CallInfoHeader, contact_header::ContactHeader,
    content_disposition_header::ContentDispositionHeader,
    content_encoding_header::ContentEncodingHeader, generic_header::GenericHeader, CSeqHeader,
    ContentLanguageHeader, ContentLengthHeader, ContentTypeHeader, ErrorInfoHeader, ExpiresHeader,
    Header, InReplyToHeader, MinExpiresHeader, OrganizationHeader, PriorityHeader,
    ProxyAuthenticateHeader, ProxyAuthorizationHeader, ProxyRequireHeader,
};

fn discrete_type(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        alt((
            tag("text"),
            tag("image"),
            tag("audio"),
            tag("video"),
            tag("application"),
        )),
        String::from_utf8_lossy,
    )(input)
}

fn composite_type(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        alt((tag("message"), tag("multipart"))),
        String::from_utf8_lossy,
    )(input)
}

#[inline]
fn ietf_token(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn x_token(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(recognize(pair(tag("x-"), token)), String::from_utf8_lossy)(input)
}

fn extension_token(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((ietf_token, x_token))(input)
}

fn m_type(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((discrete_type, composite_type, extension_token))(input)
}

#[inline]
fn iana_token(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn m_subtype(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((extension_token, iana_token))(input)
}

fn media_range(input: &[u8]) -> ParserResult<&[u8], MediaRange> {
    map(
        alt((
            separated_pair(
                map(tag("*"), String::from_utf8_lossy),
                tag("/"),
                map(tag("*"), String::from_utf8_lossy),
            ),
            separated_pair(m_type, slash, map(tag("*"), String::from_utf8_lossy)),
            separated_pair(m_type, slash, m_subtype),
        )),
        |(r#type, subtype)| MediaRange::new(r#type, subtype),
    )(input)
}

fn qvalue(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        recognize(alt((
            pair(tag("0"), opt(pair(tag("."), many_m_n(0, 3, digit)))),
            pair(tag("1"), opt(pair(tag("."), many_m_n(0, 3, tag("0"))))),
        ))),
        String::from_utf8_lossy,
    )(input)
}

fn q_param(input: &[u8]) -> ParserResult<&[u8], AcceptParameter> {
    map(separated_pair(tag("q"), equal, qvalue), |(key, value)| {
        AcceptParameter::new(String::from_utf8_lossy(key), Some(value))
    })(input)
}

fn gen_value(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    alt((
        map(token, WrappedString::new_not_wrapped),
        map(host, WrappedString::new_not_wrapped),
        quoted_string,
    ))(input)
}

fn generic_param(input: &[u8]) -> ParserResult<&[u8], GenericParameter> {
    map(
        pair(token, opt(preceded(equal, gen_value))),
        |(key, value)| GenericParameter::new(key.to_string(), value.map(|v| v.to_string())),
    )(input)
}

fn accept_param(input: &[u8]) -> ParserResult<&[u8], AcceptParameter> {
    context(
        "accept_param",
        alt((q_param, map(generic_param, Into::into))),
    )(input)
}

fn accept_range(input: &[u8]) -> ParserResult<&[u8], AcceptRange> {
    map(
        pair(media_range, many0(preceded(semi, accept_param))),
        |(media_range, accept_params)| AcceptRange::new(media_range, accept_params),
    )(input)
}

fn accept(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "accept",
        map(
            tuple((
                map(tag_no_case("Accept"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list0(comma, accept_range))),
            )),
            |(name, separator, (value, ranges))| {
                Header::Accept(AcceptHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    ranges,
                ))
            },
        ),
    )(input)
}

#[inline]
pub(crate) fn content_coding(input: &[u8]) -> ParserResult<&[u8], ContentEncoding> {
    map(token, ContentEncoding::new)(input)
}

fn codings(input: &[u8]) -> ParserResult<&[u8], ContentEncoding> {
    alt((
        content_coding,
        map(tag("*"), |v| {
            ContentEncoding::new(String::from_utf8_lossy(v))
        }),
    ))(input)
}

fn encoding(input: &[u8]) -> ParserResult<&[u8], AcceptEncoding> {
    context(
        "encoding",
        map(
            pair(codings, many0(preceded(semi, accept_param))),
            |(codings, params)| AcceptEncoding::new(codings, params),
        ),
    )(input)
}

fn accept_encoding(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "accept_encoding",
        map(
            tuple((
                map(tag_no_case("Accept-Encoding"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list0(comma, encoding))),
            )),
            |(name, separator, (value, encodings))| {
                Header::AcceptEncoding(AcceptEncodingHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    encodings,
                ))
            },
        ),
    )(input)
}

fn language_range(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    context(
        "language_range",
        map(
            alt((
                recognize(pair(
                    many_m_n(1, 8, alpha),
                    opt(many0(pair(tag("-"), many_m_n(1, 8, alpha)))),
                )),
                tag("*"),
            )),
            String::from_utf8_lossy,
        ),
    )(input)
}

fn language(input: &[u8]) -> ParserResult<&[u8], AcceptLanguage> {
    context(
        "language",
        map(
            pair(language_range, many0(preceded(semi, accept_param))),
            |(language, params)| AcceptLanguage::new(language.into_owned(), params),
        ),
    )(input)
}

fn accept_language(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "accept_language",
        map(
            tuple((
                map(tag_no_case("Accept-Language"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list0(comma, language))),
            )),
            |(name, separator, (value, languages))| {
                Header::AcceptLanguage(AcceptLanguageHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    languages,
                ))
            },
        ),
    )(input)
}

fn alert_param(input: &[u8]) -> ParserResult<&[u8], Alert> {
    context(
        "alert_param",
        map(
            pair(
                delimited(laquot, absolute_uri, raquot),
                many0(preceded(semi, map(generic_param, Into::into))),
            ),
            |(uri, params)| Alert::new(uri, params),
        ),
    )(input)
}

fn alert_info(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "alert_info",
        map(
            tuple((
                map(tag_no_case("Alert-Info"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list1(comma, alert_param))),
            )),
            |(name, separator, (value, alerts))| {
                Header::AlertInfo(AlertInfoHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    alerts,
                ))
            },
        ),
    )(input)
}

fn allow(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "allow",
        map(
            tuple((
                map(tag_no_case("Allow"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list0(comma, method))),
            )),
            |(name, separator, (value, methods))| {
                Header::Allow(AllowHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    methods,
                ))
            },
        ),
    )(input)
}

#[inline]
fn nonce_value(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    quoted_string(input)
}

fn nextnonce(input: &[u8]) -> ParserResult<&[u8], AuthenticationInfo> {
    context(
        "nextnonce",
        map(
            separated_pair(tag_no_case("nextnonce"), equal, nonce_value),
            |(_, value)| AuthenticationInfo::NextNonce(value),
        ),
    )(input)
}

fn qop_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((
        map(tag_no_case("auth-int"), String::from_utf8_lossy),
        map(tag_no_case("auth"), String::from_utf8_lossy),
        token,
    ))(input)
}

fn message_qop(input: &[u8]) -> ParserResult<&[u8], AuthenticationInfo> {
    context(
        "message_qop",
        map(
            separated_pair(tag_no_case("qop"), equal, cut(qop_value)),
            |(_, value)| AuthenticationInfo::Qop(MessageQop::new(value)),
        ),
    )(input)
}

fn response_digest(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    map(delimited(ldquot, many0(lhex), rdquot), |digits| {
        WrappedString::new_quoted(
            digits
                .iter()
                .map(|digit| String::from_utf8_lossy(digit).into_owned())
                .collect::<Vec<String>>()
                .join(""),
        )
    })(input)
}

fn response_auth(input: &[u8]) -> ParserResult<&[u8], AuthenticationInfo> {
    context(
        "response_auth",
        map(
            separated_pair(tag_no_case("rspauth"), equal, response_digest),
            |(_, value)| AuthenticationInfo::ResponseAuth(value),
        ),
    )(input)
}

#[inline]
fn cnonce_value(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    nonce_value(input)
}

fn cnonce(input: &[u8]) -> ParserResult<&[u8], AuthenticationInfo> {
    context(
        "cnonce",
        map(
            separated_pair(tag_no_case("cnonce"), equal, cut(cnonce_value)),
            |(_, value)| AuthenticationInfo::CNonce(value),
        ),
    )(input)
}

fn nc_value(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    map(count(lhex, 8), |digits| {
        WrappedString::new_not_wrapped(
            digits
                .iter()
                .map(|digit| String::from_utf8_lossy(digit).into_owned())
                .collect::<Vec<String>>()
                .join(""),
        )
    })(input)
}

fn nonce_count(input: &[u8]) -> ParserResult<&[u8], AuthenticationInfo> {
    context(
        "nonce_count",
        map(
            separated_pair(tag_no_case("nc"), equal, cut(nc_value)),
            |(_, value)| AuthenticationInfo::NonceCount(value),
        ),
    )(input)
}

fn ainfo(input: &[u8]) -> ParserResult<&[u8], AuthenticationInfo> {
    alt((nextnonce, message_qop, response_auth, cnonce, nonce_count))(input)
}

fn authentication_info(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "authentication_info",
        map(
            tuple((
                map(tag_no_case("Authentication-Info"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list1(comma, ainfo))),
            )),
            |(name, separator, (value, ainfos))| {
                Header::AuthenticationInfo(AuthenticationInfoHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    ainfos,
                ))
            },
        ),
    )(input)
}

#[inline]
fn username_value(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    quoted_string(input)
}

fn username(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    context(
        "username",
        map(
            separated_pair(tag_no_case("username"), equal, cut(username_value)),
            |(_, value)| AuthParameter::Username(value),
        ),
    )(input)
}

#[inline]
fn realm_value(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    quoted_string(input)
}

fn realm(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    context(
        "realm",
        map(
            separated_pair(tag_no_case("realm"), equal, cut(realm_value)),
            |(_, value)| AuthParameter::Realm(value),
        ),
    )(input)
}

fn nonce(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    context(
        "nonce",
        map(
            separated_pair(tag_no_case("nonce"), equal, cut(nonce_value)),
            |(_, value)| AuthParameter::Nonce(value),
        ),
    )(input)
}

fn digest_uri_value(input: &[u8]) -> ParserResult<&[u8], Uri> {
    delimited(ldquot, request_uri, rdquot)(input)
}

fn digest_uri(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    context(
        "digest_uri",
        map(
            separated_pair(tag_no_case("uri"), equal, cut(digest_uri_value)),
            |(_, value)| AuthParameter::DigestUri(value),
        ),
    )(input)
}

fn request_digest(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    context(
        "request_digest",
        map(
            delimited(ldquot, recognize(many_m_n(32, 32, lhex)), rdquot),
            |v| WrappedString::new_quoted(String::from_utf8_lossy(v)),
        ),
    )(input)
}

fn dresponse(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    context(
        "dresponse",
        map(
            separated_pair(tag_no_case("response"), equal, cut(request_digest)),
            |(_, value)| AuthParameter::DResponse(value),
        ),
    )(input)
}

fn algorithm(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    context(
        "algorithm",
        map(
            separated_pair(
                tag_no_case("algorithm"),
                equal,
                cut(alt((
                    map(tag_no_case("MD5"), String::from_utf8_lossy),
                    map(tag_no_case("MD5-sess"), String::from_utf8_lossy),
                    token,
                ))),
            ),
            |(_, value)| AuthParameter::Algorithm(Algorithm::new(value)),
        ),
    )(input)
}

fn opaque(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    context(
        "opaque",
        map(
            separated_pair(tag_no_case("opaque"), equal, cut(quoted_string)),
            |(_, value)| AuthParameter::Opaque(value),
        ),
    )(input)
}

#[inline]
fn auth_param_name(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn auth_param(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    context(
        "auth_param",
        map(
            separated_pair(
                auth_param_name,
                equal,
                alt((map(token, WrappedString::new_not_wrapped), quoted_string)),
            ),
            |(key, value)| AuthParameter::Other(key.to_string(), value),
        ),
    )(input)
}

fn dig_resp(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    context(
        "dig_resp",
        alt((
            username,
            realm,
            nonce,
            digest_uri,
            dresponse,
            algorithm,
            map(cnonce, |ainfo| ainfo.try_into().unwrap()),
            opaque,
            map(message_qop, |ainfo| ainfo.try_into().unwrap()),
            map(nonce_count, |ainfo| ainfo.try_into().unwrap()),
            auth_param,
        )),
    )(input)
}

fn digest_response(input: &[u8]) -> ParserResult<&[u8], AuthParameters> {
    context(
        "digest_response",
        map(separated_list1(comma, dig_resp), Into::into),
    )(input)
}

#[inline]
fn auth_scheme(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn auth_params(input: &[u8]) -> ParserResult<&[u8], AuthParameters> {
    map(separated_list1(comma, auth_param), Into::into)(input)
}

fn digest_credentials(input: &[u8]) -> ParserResult<&[u8], Credentials> {
    context(
        "digest_credentials",
        map(
            separated_pair(
                map(tag_no_case("Digest"), String::from_utf8_lossy),
                lws,
                cut(digest_response),
            ),
            |(_, params)| Credentials::Digest(params),
        ),
    )(input)
}

fn other_response(input: &[u8]) -> ParserResult<&[u8], Credentials> {
    context(
        "other_response",
        map(
            separated_pair(auth_scheme, lws, auth_params),
            |(scheme, params)| Credentials::Other(scheme.to_string(), params),
        ),
    )(input)
}

fn credentials(input: &[u8]) -> ParserResult<&[u8], Credentials> {
    context("credentials", alt((digest_credentials, other_response)))(input)
}

fn authorization(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "authorization",
        map(
            tuple((
                map(tag_no_case("Authorization"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(credentials)),
            )),
            |(name, separator, (value, credentials))| {
                Header::Authorization(AuthorizationHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    credentials,
                ))
            },
        ),
    )(input)
}

fn callid(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        recognize(pair(word, opt(pair(tag("@"), word)))),
        String::from_utf8_lossy,
    )(input)
}

fn call_id(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "call_id",
        map(
            tuple((
                map(
                    alt((tag_no_case("Call-ID"), tag_no_case("i"))),
                    String::from_utf8_lossy,
                ),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(callid)),
            )),
            |(name, separator, (value, call_id))| {
                Header::CallId(CallIdHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    call_id,
                ))
            },
        ),
    )(input)
}

fn info_param(input: &[u8]) -> ParserResult<&[u8], CallInfoParameter> {
    map(
        alt((
            map(
                separated_pair(
                    map(tag_no_case("purpose"), String::from_utf8_lossy),
                    equal,
                    map(
                        alt((
                            map(tag_no_case("icon"), String::from_utf8_lossy),
                            map(tag_no_case("info"), String::from_utf8_lossy),
                            map(tag_no_case("card"), String::from_utf8_lossy),
                            token,
                        )),
                        Some,
                    ),
                ),
                |(key, value)| GenericParameter::new(key.to_string(), value.map(Into::into)),
            ),
            generic_param,
        )),
        Into::into,
    )(input)
}

fn info(input: &[u8]) -> ParserResult<&[u8], CallInfo> {
    map(
        tuple((
            laquot,
            absolute_uri,
            raquot,
            many0(preceded(semi, info_param)),
        )),
        |(_, uri, _, params)| CallInfo::new(uri, params),
    )(input)
}

fn call_info(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "call_info",
        map(
            tuple((
                map(tag_no_case("Call-Info"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list1(comma, info))),
            )),
            |(name, separator, (value, infos))| {
                Header::CallInfo(CallInfoHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    infos,
                ))
            },
        ),
    )(input)
}

fn addr_spec(input: &[u8]) -> ParserResult<&[u8], Uri> {
    alt((sip_uri, map(absolute_uri, Uri::Absolute)))(input)
}

fn display_name(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    alt((
        quoted_string,
        map(recognize(many0(pair(token, lws))), |v| {
            WrappedString::new_not_wrapped(String::from_utf8_lossy(v).trim_end())
        }),
    ))(input)
}

fn name_addr(input: &[u8]) -> ParserResult<&[u8], NameAddress> {
    map(
        pair(opt(display_name), delimited(laquot, addr_spec, raquot)),
        |(display_name, uri)| NameAddress::new(uri, display_name),
    )(input)
}

fn c_p_q(input: &[u8]) -> ParserResult<&[u8], ContactParameter> {
    map(
        separated_pair(
            map(tag_no_case("q"), String::from_utf8_lossy),
            equal,
            qvalue,
        ),
        |(_, value)| ContactParameter::Q(value.to_string()),
    )(input)
}

#[inline]
fn delta_seconds(input: &[u8]) -> ParserResult<&[u8], u32> {
    map(recognize(many1(digit)), |digits| {
        String::from_utf8_lossy(digits)
            .parse::<u32>()
            .unwrap_or(u32::MAX)
    })(input)
}

fn c_p_expires(input: &[u8]) -> ParserResult<&[u8], ContactParameter> {
    map(
        separated_pair(
            tag_no_case("expires"),
            equal,
            map(delta_seconds, |seconds| seconds.to_string()),
        ),
        |(_, value)| ContactParameter::Expires(value),
    )(input)
}

#[inline]
fn contact_extension(input: &[u8]) -> ParserResult<&[u8], GenericParameter> {
    generic_param(input)
}

fn contact_params(input: &[u8]) -> ParserResult<&[u8], ContactParameter> {
    alt((c_p_q, c_p_expires, map(contact_extension, Into::into)))(input)
}

fn contact_param(input: &[u8]) -> ParserResult<&[u8], Contact> {
    map(
        pair(
            alt((name_addr, map(addr_spec, |uri| NameAddress::new(uri, None)))),
            many0(preceded(semi, contact_params)),
        ),
        |(address, params)| Contact::new(address, params),
    )(input)
}

fn contact(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "contact",
        map(
            tuple((
                map(
                    alt((tag_no_case("Contact"), tag_no_case("m"))),
                    String::from_utf8_lossy,
                ),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(alt((
                    map(star, |_| Contacts::Any),
                    map(separated_list1(comma, contact_param), Contacts::Contacts),
                )))),
            )),
            |(name, separator, (value, contacts))| {
                Header::Contact(ContactHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    contacts,
                ))
            },
        ),
    )(input)
}

#[inline]
fn disp_extension_token(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn disp_type(input: &[u8]) -> ParserResult<&[u8], DispositionType> {
    map(
        alt((
            map(
                alt((
                    tag_no_case("render"),
                    tag_no_case("session"),
                    tag_no_case("icon"),
                    tag_no_case("alert"),
                )),
                String::from_utf8_lossy,
            ),
            disp_extension_token,
        )),
        DispositionType::new,
    )(input)
}

#[inline]
fn other_handling(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn handling_param(input: &[u8]) -> ParserResult<&[u8], DispositionParameter> {
    map(
        separated_pair(
            tag_no_case("handling"),
            equal,
            map(
                alt((
                    map(tag_no_case("optional"), String::from_utf8_lossy),
                    map(tag_no_case("required"), String::from_utf8_lossy),
                    other_handling,
                )),
                Handling::new,
            ),
        ),
        |(_, value)| DispositionParameter::Handling(value),
    )(input)
}

fn disp_param(input: &[u8]) -> ParserResult<&[u8], DispositionParameter> {
    alt((handling_param, map(generic_param, Into::into)))(input)
}

fn content_disposition(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "content_disposition",
        map(
            tuple((
                map(tag_no_case("Content-Disposition"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(pair(disp_type, many0(preceded(semi, disp_param))))),
            )),
            |(name, separator, (value, (r#type, params)))| {
                Header::ContentDisposition(ContentDispositionHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    r#type,
                    params,
                ))
            },
        ),
    )(input)
}

fn content_encoding(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "content_encoding",
        map(
            tuple((
                map(
                    alt((tag_no_case("Content-Encoding"), tag("e"))),
                    String::from_utf8_lossy,
                ),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list1(comma, content_coding))),
            )),
            |(name, separator, (value, encodings))| {
                Header::ContentEncoding(ContentEncodingHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    encodings,
                ))
            },
        ),
    )(input)
}

#[inline]
fn primary_tag(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(many_m_n(1, 8, alpha))(input)
}

#[inline]
fn subtag(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    primary_tag(input)
}

pub(crate) fn language_tag(input: &[u8]) -> ParserResult<&[u8], ContentLanguage> {
    map(
        recognize(pair(primary_tag, many0(preceded(tag("-"), subtag)))),
        |value| ContentLanguage::new(String::from_utf8_lossy(value)),
    )(input)
}

fn content_language(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "content_language",
        map(
            tuple((
                map(tag_no_case("Content-Language"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list1(comma, language_tag))),
            )),
            |(name, separator, (value, languages))| {
                Header::ContentLanguage(ContentLanguageHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    languages,
                ))
            },
        ),
    )(input)
}

fn content_length(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "content_length",
        map(
            tuple((
                map(
                    alt((tag_no_case("Content-Length"), tag_no_case("l"))),
                    String::from_utf8_lossy,
                ),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(map(recognize(many1(digit)), |l| {
                    String::from_utf8_lossy(l).parse::<u32>().unwrap()
                }))),
            )),
            |(name, separator, (value, content_length))| {
                Header::ContentLength(ContentLengthHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    content_length,
                ))
            },
        ),
    )(input)
}

#[inline]
fn m_attribute(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn m_value(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    alt((map(token, WrappedString::new_not_wrapped), quoted_string))(input)
}

fn m_parameter(input: &[u8]) -> ParserResult<&[u8], MediaParameter> {
    map(
        separated_pair(m_attribute, equal, m_value),
        |(key, value)| MediaParameter::new(key, value),
    )(input)
}

fn media_type(input: &[u8]) -> ParserResult<&[u8], MediaType> {
    map(
        tuple((m_type, slash, m_subtype, many0(preceded(semi, m_parameter)))),
        |(r#type, _, subtype, parameters)| {
            MediaType::new(MediaRange::new(r#type, subtype), parameters)
        },
    )(input)
}

fn content_type(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "content_type",
        map(
            tuple((
                map(
                    alt((tag_no_case("Content-Type"), tag_no_case("c"))),
                    String::from_utf8_lossy,
                ),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(media_type)),
            )),
            |(name, separator, (value, media_type))| {
                Header::ContentType(ContentTypeHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    media_type,
                ))
            },
        ),
    )(input)
}

fn cseq(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "cseq",
        map(
            tuple((
                map(tag_no_case("CSeq"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_pair(
                    map(recognize(many1(digit)), |cseq| {
                        String::from_utf8_lossy(cseq).parse::<u32>().unwrap()
                    }),
                    lws,
                    method,
                ))),
            )),
            |(name, separator, (value, (cseq, method)))| {
                Header::CSeq(CSeqHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    cseq,
                    method,
                ))
            },
        ),
    )(input)
}

fn wkday(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        alt((
            tag("Mon"),
            tag("Tue"),
            tag("Wed"),
            tag("Thu"),
            tag("Fri"),
            tag("Sat"),
            tag("Sun"),
        )),
        String::from_utf8_lossy,
    )(input)
}

fn month(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        alt((
            tag("Jan"),
            tag("Feb"),
            tag("Mar"),
            tag("Apr"),
            tag("May"),
            tag("Jun"),
            tag("Jul"),
            tag("Aug"),
            tag("Sep"),
            tag("Oct"),
            tag("Nov"),
            tag("Dec"),
        )),
        String::from_utf8_lossy,
    )(input)
}

fn date1(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        recognize(tuple((count(digit, 2), sp, month, sp, count(digit, 4)))),
        String::from_utf8_lossy,
    )(input)
}

fn time(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        recognize(tuple((
            count(digit, 2),
            tag(":"),
            count(digit, 2),
            tag(":"),
            count(digit, 2),
        ))),
        String::from_utf8_lossy,
    )(input)
}

fn rfc1123_date(input: &[u8]) -> ParserResult<&[u8], DateTime<Utc>> {
    let result = map(
        recognize(tuple((
            wkday,
            tag(","),
            sp,
            date1,
            sp,
            time,
            sp,
            tag("GMT"),
        ))),
        String::from_utf8_lossy,
    )(input);
    match result {
        Err(e) => Err(e),
        Ok((rest, date)) => {
            let result = DateTime::parse_from_rfc2822(date.as_ref());
            match result {
                Ok(date) => Ok((rest, date.to_utc())),
                Err(_) => Err(nom::Err::Error(VerboseError::from_error_kind(
                    input,
                    ErrorKind::Verify,
                ))),
            }
        }
    }
}

#[inline]
fn sip_date(input: &[u8]) -> ParserResult<&[u8], DateTime<Utc>> {
    rfc1123_date(input)
}

fn date(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "date",
        map(
            tuple((
                map(tag_no_case("Date"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(sip_date)),
            )),
            |(name, separator, (value, date))| {
                Header::Date(DateHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    date,
                ))
            },
        ),
    )(input)
}

fn error_uri(input: &[u8]) -> ParserResult<&[u8], ErrorUri> {
    map(
        tuple((
            laquot,
            request_uri,
            raquot,
            many0(preceded(semi, generic_param)),
        )),
        |(_, uri, _, parameters)| ErrorUri::new(uri, parameters),
    )(input)
}

fn error_info(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "error_info",
        map(
            tuple((
                map(tag_no_case("Error-Info"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list1(comma, error_uri))),
            )),
            |(name, separator, (value, uris))| {
                Header::ErrorInfo(ErrorInfoHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    uris,
                ))
            },
        ),
    )(input)
}

fn expires(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "expires",
        map(
            tuple((
                map(tag_no_case("Expires"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(delta_seconds)),
            )),
            |(name, separator, (value, expires))| {
                Header::Expires(ExpiresHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    expires,
                ))
            },
        ),
    )(input)
}

fn tag_param(input: &[u8]) -> ParserResult<&[u8], GenericParameter> {
    map(
        separated_pair(
            map(tag_no_case("tag"), String::from_utf8_lossy),
            equal,
            token,
        ),
        |(key, value)| GenericParameter::new(key, Some(value)),
    )(input)
}

fn from_param(input: &[u8]) -> ParserResult<&[u8], FromParameter> {
    map(alt((tag_param, generic_param)), Into::into)(input)
}

fn from_spec(input: &[u8]) -> ParserResult<&[u8], (NameAddress, Vec<FromParameter>)> {
    pair(
        alt((map(addr_spec, |uri| NameAddress::new(uri, None)), name_addr)),
        many0(preceded(semi, from_param)),
    )(input)
}

fn from(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "from",
        map(
            tuple((
                map(
                    alt((tag_no_case("From"), tag_no_case("f"))),
                    String::from_utf8_lossy,
                ),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(from_spec)),
            )),
            |(name, separator, (value, (address, parameters)))| {
                Header::From(FromHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    address,
                    parameters,
                ))
            },
        ),
    )(input)
}

fn in_reply_to(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "in_reply_to",
        map(
            tuple((
                map(tag_no_case("In-Reply-To"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list1(comma, callid))),
            )),
            |(name, separator, (value, call_ids))| {
                Header::InReplyTo(InReplyToHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    call_ids,
                ))
            },
        ),
    )(input)
}

fn max_forwards(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "max_forwards",
        map(
            tuple((
                map(tag_no_case("Max-Forwards"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(map(recognize(many1(digit)), |value| {
                    String::from_utf8_lossy(value)
                        .parse::<u8>()
                        .unwrap_or(u8::MAX)
                }))),
            )),
            |(name, separator, (value, max_forwards))| {
                Header::MaxForwards(MaxForwardsHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    max_forwards,
                ))
            },
        ),
    )(input)
}

fn mime_version(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "mime_version",
        map(
            tuple((
                map(tag_no_case("MIME-Version"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(map(
                    recognize(tuple((many1(digit), tag("."), many1(digit)))),
                    String::from_utf8_lossy,
                ))),
            )),
            |(name, separator, (value, version))| {
                Header::MimeVersion(MimeVersionHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    version,
                ))
            },
        ),
    )(input)
}

fn min_expires(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "min_expires",
        map(
            tuple((
                map(tag_no_case("Min-Expires"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(delta_seconds)),
            )),
            |(name, separator, (value, min_expires))| {
                Header::MinExpires(MinExpiresHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    min_expires,
                ))
            },
        ),
    )(input)
}

fn organization(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "organization",
        map(
            tuple((
                map(tag_no_case("Organization"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(opt(text_utf8_trim))),
            )),
            |(name, separator, (value, organization))| {
                Header::Organization(OrganizationHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    organization.unwrap_or_default(),
                ))
            },
        ),
    )(input)
}

#[inline]
fn other_priority(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn priority_value(input: &[u8]) -> ParserResult<&[u8], Priority> {
    map(
        alt((
            map(tag_no_case("emergency"), String::from_utf8_lossy),
            map(tag_no_case("urgent"), String::from_utf8_lossy),
            map(tag_no_case("normal"), String::from_utf8_lossy),
            map(tag_no_case("non-urgent"), String::from_utf8_lossy),
            other_priority,
        )),
        Priority::new,
    )(input)
}

fn priority(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "priority",
        map(
            tuple((
                map(tag_no_case("Priority"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(priority_value)),
            )),
            |(name, separator, (value, priority))| {
                Header::Priority(PriorityHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    priority,
                ))
            },
        ),
    )(input)
}

fn other_challenge(input: &[u8]) -> ParserResult<&[u8], Challenge> {
    map(
        separated_pair(auth_scheme, lws, separated_list1(comma, auth_param)),
        |(scheme, auth_params)| Challenge::Other(scheme.to_string(), auth_params.into()),
    )(input)
}

fn segment(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        recognize(pair(many0(pchar), many0(preceded(tag(";"), param)))),
        String::from_utf8_lossy,
    )(input)
}

fn path_segments(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        recognize(pair(segment, many0(preceded(tag("/"), segment)))),
        String::from_utf8_lossy,
    )(input)
}

fn abs_path(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        recognize(pair(tag("/"), path_segments)),
        String::from_utf8_lossy,
    )(input)
}

fn uri(input: &[u8]) -> ParserResult<&[u8], DomainUri> {
    alt((
        map(request_uri, DomainUri::Uri),
        map(abs_path, |path| DomainUri::AbsPath(path.to_string())),
    ))(input)
}

fn domain_value(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    delimited(
        ldquot,
        map(separated_list1(many1(sp), uri), |uris| {
            AuthParameter::Domain(uris.into())
        }),
        rdquot,
    )(input)
}

fn domain(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        tuple((tag_no_case("domain"), equal, cut(domain_value))),
        |(_, _, domain)| domain,
    )(input)
}

fn stale(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(
            tag_no_case("stale"),
            equal,
            cut(alt((
                map(tag_no_case("true"), |v| {
                    AuthParameter::Stale(Stale::new(String::from_utf8_lossy(v), true))
                }),
                map(tag_no_case("false"), |v| {
                    AuthParameter::Stale(Stale::new(String::from_utf8_lossy(v), false))
                }),
            ))),
        ),
        |(_, stale)| stale,
    )(input)
}

fn qop_options(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(
            tag_no_case("qop"),
            equal,
            cut(delimited(
                ldquot,
                separated_list1(tag(","), qop_value),
                rdquot,
            )),
        ),
        |(_, values)| {
            AuthParameter::QopOptions(
                values
                    .iter()
                    .map(|v| MessageQop::new(v.to_string()))
                    .collect::<Vec<MessageQop>>()
                    .into(),
            )
        },
    )(input)
}

fn digest_cln(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    alt((
        realm,
        domain,
        nonce,
        opaque,
        stale,
        algorithm,
        qop_options,
        auth_param,
    ))(input)
}

fn challenge(input: &[u8]) -> ParserResult<&[u8], Challenge> {
    alt((
        map(
            separated_pair(
                tag_no_case("Digest"),
                lws,
                cut(separated_list1(comma, digest_cln)),
            ),
            |(_, auth_params)| Challenge::Digest(auth_params.into()),
        ),
        other_challenge,
    ))(input)
}

fn proxy_authenticate(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "proxy_authenticate",
        map(
            tuple((
                map(tag_no_case("Proxy-Authenticate"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(challenge)),
            )),
            |(name, separator, (value, challenge))| {
                Header::ProxyAuthenticate(ProxyAuthenticateHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    challenge,
                ))
            },
        ),
    )(input)
}

fn proxy_authorization(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "Proxy-Authorization header",
        map(
            tuple((
                map(tag_no_case("Proxy-Authorization"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(credentials)),
            )),
            |(name, separator, (value, credentials))| {
                Header::ProxyAuthorization(ProxyAuthorizationHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    credentials,
                ))
            },
        ),
    )(input)
}

pub(crate) fn option_tag(input: &[u8]) -> ParserResult<&[u8], OptionTag> {
    context("option_tag", map(token, OptionTag::new))(input)
}

fn proxy_require(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "Proxy-Require",
        map(
            tuple((
                map(tag_no_case("Proxy-Require"), String::from_utf8_lossy),
                map(hcolon, String::from_utf8_lossy),
                cut(consumed(separated_list1(comma, option_tag))),
            )),
            |(name, separator, (value, tags))| {
                Header::ProxyRequire(ProxyRequireHeader::new(
                    GenericHeader::new(name, separator, String::from_utf8_lossy(value)),
                    tags,
                ))
            },
        ),
    )(input)
}

#[inline]
fn header_name(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn header_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        recognize(many0(alt((text_utf8char, utf8_cont, lws)))),
        String::from_utf8_lossy,
    )(input)
}

fn extension_header(input: &[u8]) -> ParserResult<&[u8], Header> {
    map(
        tuple((
            header_name,
            map(hcolon, String::from_utf8_lossy),
            header_value,
        )),
        |(name, separator, value)| {
            Header::ExtensionHeader(GenericHeader::new(name, separator, value))
        },
    )(input)
}

pub(super) fn message_header(input: &[u8]) -> ParserResult<&[u8], Header> {
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
                extension_header,
            )),
        )),
    )(input)
}
