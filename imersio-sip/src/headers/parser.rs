use chrono::{DateTime, Utc};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::{consumed, cut, map, opt, recognize, value},
    error::{context, ErrorKind, ParseError, VerboseError},
    multi::{count, many0, many1, many_m_n, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};

use crate::{
    common::method::parser::method,
    common::wrapped_string::WrappedString,
    headers::{
        AcceptEncodingHeader, AcceptHeader, AcceptLanguageHeader, AlertInfoHeader, AllowHeader,
        AuthenticationInfoHeader, AuthorizationHeader, CSeqHeader, CallIdHeader, CallInfoHeader,
        ContactHeader, ContentDispositionHeader, ContentEncodingHeader, ContentLanguageHeader,
        ContentLengthHeader, ContentTypeHeader, DateHeader, ErrorInfoHeader, ExpiresHeader,
        FromHeader, GenericHeader, Header, InReplyToHeader, MaxForwardsHeader, MimeVersionHeader,
        MinExpiresHeader, OrganizationHeader, PriorityHeader, ProxyAuthenticateHeader,
        ProxyAuthorizationHeader, ProxyRequireHeader, RecordRouteHeader,
    },
    parser::{
        alpha, comma, digit, equal, hcolon, laquot, ldquot, lhex, lws, param, pchar, quoted_string,
        raquot, rdquot, semi, slash, sp, star, text_utf8_trim, text_utf8char, token, word,
        ParserResult,
    },
    uri::parser::{absolute_uri, host, request_uri, sip_uri},
    AcceptEncoding, AcceptLanguage, AcceptParameter, AcceptRange, Alert, Algorithm, AuthParameter,
    AuthParameters, AuthenticationInfo, CallId, CallInfo, CallInfoParameter, Challenge, Contact,
    ContactParameter, Contacts, ContentEncoding, ContentLanguage, Credentials,
    DispositionParameter, DispositionType, DomainUri, ErrorUri, FromParameter, GenericParameter,
    Handling, MediaParameter, MediaRange, MediaType, MessageQop, NameAddress, OptionTag, Priority,
    Route, Stale, Uri,
};

fn discrete_type(input: &str) -> ParserResult<&str, &str> {
    alt((
        tag("text"),
        tag("image"),
        tag("audio"),
        tag("video"),
        tag("application"),
    ))(input)
}

fn composite_type(input: &str) -> ParserResult<&str, &str> {
    alt((tag("message"), tag("multipart")))(input)
}

#[inline]
fn ietf_token(input: &str) -> ParserResult<&str, &str> {
    token(input)
}

fn x_token(input: &str) -> ParserResult<&str, &str> {
    recognize(pair(tag("x-"), token))(input)
}

fn extension_token(input: &str) -> ParserResult<&str, &str> {
    alt((ietf_token, x_token))(input)
}

fn m_type(input: &str) -> ParserResult<&str, &str> {
    alt((discrete_type, composite_type, extension_token))(input)
}

#[inline]
fn iana_token(input: &str) -> ParserResult<&str, &str> {
    token(input)
}

fn m_subtype(input: &str) -> ParserResult<&str, &str> {
    alt((extension_token, iana_token))(input)
}

fn media_range(input: &str) -> ParserResult<&str, MediaRange> {
    map(
        alt((
            separated_pair(tag("*"), slash, tag("*")),
            separated_pair(m_type, slash, tag("*")),
            separated_pair(m_type, slash, m_subtype),
        )),
        |(r#type, subtype)| MediaRange::new(r#type, subtype),
    )(input)
}

fn qvalue(input: &str) -> ParserResult<&str, &str> {
    context(
        "qvalue",
        recognize(alt((
            pair(
                tag("0"),
                opt(pair(tag("."), many_m_n(0, 3, recognize(digit)))),
            ),
            pair(tag("1"), opt(pair(tag("."), many_m_n(0, 3, tag("0"))))),
        ))),
    )(input)
}

fn q_param(input: &str) -> ParserResult<&str, AcceptParameter> {
    context(
        "q_param",
        map(separated_pair(tag("q"), equal, qvalue), |(key, value)| {
            AcceptParameter::new(key, Some(value))
        }),
    )(input)
}

fn gen_value(input: &str) -> ParserResult<&str, WrappedString> {
    context(
        "gen_value",
        alt((
            map(token, WrappedString::new_not_wrapped),
            map(host, WrappedString::new_not_wrapped),
            quoted_string,
        )),
    )(input)
}

fn generic_param(input: &str) -> ParserResult<&str, GenericParameter> {
    context(
        "generic_param",
        map(
            pair(token, opt(preceded(equal, gen_value))),
            |(key, value)| GenericParameter::new(key.to_string(), value.map(|v| v.to_string())),
        ),
    )(input)
}

fn accept_param(input: &str) -> ParserResult<&str, AcceptParameter> {
    context(
        "accept_param",
        alt((q_param, map(generic_param, Into::into))),
    )(input)
}

fn accept_range(input: &str) -> ParserResult<&str, AcceptRange> {
    context(
        "accept_range",
        map(
            pair(media_range, many0(preceded(semi, accept_param))),
            |(media_range, accept_params)| AcceptRange::new(media_range, accept_params),
        ),
    )(input)
}

fn accept(input: &str) -> ParserResult<&str, Header> {
    context(
        "Accept header",
        map(
            tuple((
                tag_no_case("Accept"),
                hcolon,
                cut(consumed(separated_list0(comma, accept_range))),
            )),
            |(name, separator, (value, ranges))| {
                Header::Accept(AcceptHeader::new(
                    GenericHeader::new(name, separator, value),
                    ranges,
                ))
            },
        ),
    )(input)
}

#[inline]
pub(crate) fn content_coding(input: &str) -> ParserResult<&str, ContentEncoding> {
    context("content_coding", map(token, ContentEncoding::new))(input)
}

fn codings(input: &str) -> ParserResult<&str, ContentEncoding> {
    context(
        "codings",
        alt((content_coding, map(tag("*"), ContentEncoding::new))),
    )(input)
}

fn encoding(input: &str) -> ParserResult<&str, AcceptEncoding> {
    context(
        "encoding",
        map(
            pair(codings, many0(preceded(semi, accept_param))),
            |(codings, params)| AcceptEncoding::new(codings, params),
        ),
    )(input)
}

fn accept_encoding(input: &str) -> ParserResult<&str, Header> {
    context(
        "Accept-Encoding header",
        map(
            tuple((
                tag_no_case("Accept-Encoding"),
                hcolon,
                cut(consumed(separated_list0(comma, encoding))),
            )),
            |(name, separator, (value, encodings))| {
                Header::AcceptEncoding(AcceptEncodingHeader::new(
                    GenericHeader::new(name, separator, value),
                    encodings,
                ))
            },
        ),
    )(input)
}

fn language_range(input: &str) -> ParserResult<&str, &str> {
    context(
        "language_range",
        alt((
            recognize(pair(
                many_m_n(1, 8, alpha),
                opt(many0(pair(tag("-"), many_m_n(1, 8, alpha)))),
            )),
            tag("*"),
        )),
    )(input)
}

fn language(input: &str) -> ParserResult<&str, AcceptLanguage> {
    context(
        "language",
        map(
            pair(language_range, many0(preceded(semi, accept_param))),
            |(language, params)| AcceptLanguage::new(language, params),
        ),
    )(input)
}

fn accept_language(input: &str) -> ParserResult<&str, Header> {
    context(
        "Accept-Language header",
        map(
            tuple((
                tag_no_case("Accept-Language"),
                hcolon,
                cut(consumed(separated_list0(comma, language))),
            )),
            |(name, separator, (value, languages))| {
                Header::AcceptLanguage(AcceptLanguageHeader::new(
                    GenericHeader::new(name, separator, value),
                    languages,
                ))
            },
        ),
    )(input)
}

fn alert_param(input: &str) -> ParserResult<&str, Alert> {
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

fn alert_info(input: &str) -> ParserResult<&str, Header> {
    context(
        "Alert-Info header",
        map(
            tuple((
                tag_no_case("Alert-Info"),
                hcolon,
                cut(consumed(separated_list1(comma, alert_param))),
            )),
            |(name, separator, (value, alerts))| {
                Header::AlertInfo(AlertInfoHeader::new(
                    GenericHeader::new(name, separator, value),
                    alerts,
                ))
            },
        ),
    )(input)
}

fn allow(input: &str) -> ParserResult<&str, Header> {
    context(
        "Allow header",
        map(
            tuple((
                tag_no_case("Allow"),
                hcolon,
                cut(consumed(separated_list0(comma, method))),
            )),
            |(name, separator, (value, methods))| {
                Header::Allow(AllowHeader::new(
                    GenericHeader::new(name, separator, value),
                    methods,
                ))
            },
        ),
    )(input)
}

#[inline]
fn nonce_value(input: &str) -> ParserResult<&str, WrappedString> {
    quoted_string(input)
}

fn nextnonce(input: &str) -> ParserResult<&str, AuthenticationInfo> {
    context(
        "nextnonce",
        map(
            separated_pair(tag_no_case("nextnonce"), equal, nonce_value),
            |(_, value)| AuthenticationInfo::NextNonce(value),
        ),
    )(input)
}

fn qop_value(input: &str) -> ParserResult<&str, &str> {
    alt((tag_no_case("auth-int"), tag_no_case("auth"), token))(input)
}

fn message_qop(input: &str) -> ParserResult<&str, AuthenticationInfo> {
    context(
        "message_qop",
        map(
            separated_pair(tag_no_case("qop"), equal, cut(qop_value)),
            |(_, value)| AuthenticationInfo::Qop(MessageQop::new(value)),
        ),
    )(input)
}

fn response_digest(input: &str) -> ParserResult<&str, WrappedString> {
    map(delimited(ldquot, many0(lhex), rdquot), |digits| {
        WrappedString::new_quoted(
            digits
                .into_iter()
                .map(Into::into)
                .collect::<Vec<String>>()
                .join(""),
        )
    })(input)
}

fn response_auth(input: &str) -> ParserResult<&str, AuthenticationInfo> {
    context(
        "response_auth",
        map(
            separated_pair(tag_no_case("rspauth"), equal, response_digest),
            |(_, value)| AuthenticationInfo::ResponseAuth(value),
        ),
    )(input)
}

#[inline]
fn cnonce_value(input: &str) -> ParserResult<&str, WrappedString> {
    nonce_value(input)
}

fn cnonce(input: &str) -> ParserResult<&str, AuthenticationInfo> {
    context(
        "cnonce",
        map(
            separated_pair(tag_no_case("cnonce"), equal, cut(cnonce_value)),
            |(_, value)| AuthenticationInfo::CNonce(value),
        ),
    )(input)
}

fn nc_value(input: &str) -> ParserResult<&str, WrappedString> {
    map(count(lhex, 8), |digits| {
        WrappedString::new_not_wrapped(
            digits
                .into_iter()
                .map(Into::into)
                .collect::<Vec<String>>()
                .join(""),
        )
    })(input)
}

fn nonce_count(input: &str) -> ParserResult<&str, AuthenticationInfo> {
    context(
        "nonce_count",
        map(
            separated_pair(tag_no_case("nc"), equal, cut(nc_value)),
            |(_, value)| AuthenticationInfo::NonceCount(value),
        ),
    )(input)
}

fn ainfo(input: &str) -> ParserResult<&str, AuthenticationInfo> {
    alt((nextnonce, message_qop, response_auth, cnonce, nonce_count))(input)
}

fn authentication_info(input: &str) -> ParserResult<&str, Header> {
    context(
        "Authentication-Info header",
        map(
            tuple((
                tag_no_case("Authentication-Info"),
                hcolon,
                cut(consumed(separated_list1(comma, ainfo))),
            )),
            |(name, separator, (value, ainfos))| {
                Header::AuthenticationInfo(AuthenticationInfoHeader::new(
                    GenericHeader::new(name, separator, value),
                    ainfos,
                ))
            },
        ),
    )(input)
}

#[inline]
fn username_value(input: &str) -> ParserResult<&str, WrappedString> {
    quoted_string(input)
}

fn username(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "username",
        map(
            separated_pair(tag_no_case("username"), equal, cut(username_value)),
            |(_, value)| AuthParameter::Username(value),
        ),
    )(input)
}

#[inline]
fn realm_value(input: &str) -> ParserResult<&str, WrappedString> {
    quoted_string(input)
}

fn realm(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "realm",
        map(
            separated_pair(tag_no_case("realm"), equal, cut(realm_value)),
            |(_, value)| AuthParameter::Realm(value),
        ),
    )(input)
}

fn nonce(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "nonce",
        map(
            separated_pair(tag_no_case("nonce"), equal, cut(nonce_value)),
            |(_, value)| AuthParameter::Nonce(value),
        ),
    )(input)
}

fn digest_uri_value(input: &str) -> ParserResult<&str, Uri> {
    delimited(ldquot, request_uri, rdquot)(input)
}

fn digest_uri(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "digest_uri",
        map(
            separated_pair(tag_no_case("uri"), equal, cut(digest_uri_value)),
            |(_, value)| AuthParameter::DigestUri(value),
        ),
    )(input)
}

fn request_digest(input: &str) -> ParserResult<&str, WrappedString> {
    context(
        "request_digest",
        map(
            delimited(ldquot, recognize(many_m_n(32, 32, lhex)), rdquot),
            WrappedString::new_quoted,
        ),
    )(input)
}

fn dresponse(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "dresponse",
        map(
            separated_pair(tag_no_case("response"), equal, cut(request_digest)),
            |(_, value)| AuthParameter::DResponse(value),
        ),
    )(input)
}

fn algorithm(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "algorithm",
        map(
            separated_pair(
                tag_no_case("algorithm"),
                equal,
                cut(alt((tag_no_case("MD5"), tag_no_case("MD5-sess"), token))),
            ),
            |(_, value)| AuthParameter::Algorithm(Algorithm::new(value)),
        ),
    )(input)
}

fn opaque(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "opaque",
        map(
            separated_pair(tag_no_case("opaque"), equal, cut(quoted_string)),
            |(_, value)| AuthParameter::Opaque(value),
        ),
    )(input)
}

#[inline]
fn auth_param_name(input: &str) -> ParserResult<&str, &str> {
    token(input)
}

fn auth_param(input: &str) -> ParserResult<&str, AuthParameter> {
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

fn dig_resp(input: &str) -> ParserResult<&str, AuthParameter> {
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

fn digest_response(input: &str) -> ParserResult<&str, AuthParameters> {
    context(
        "digest_response",
        map(separated_list1(comma, dig_resp), Into::into),
    )(input)
}

#[inline]
fn auth_scheme(input: &str) -> ParserResult<&str, &str> {
    token(input)
}

fn auth_params(input: &str) -> ParserResult<&str, AuthParameters> {
    context(
        "auth_params",
        map(separated_list1(comma, auth_param), Into::into),
    )(input)
}

fn digest_credentials(input: &str) -> ParserResult<&str, Credentials> {
    context(
        "digest_credentials",
        map(
            separated_pair(tag_no_case("Digest"), lws, cut(digest_response)),
            |(_, params)| Credentials::Digest(params),
        ),
    )(input)
}

fn other_response(input: &str) -> ParserResult<&str, Credentials> {
    context(
        "other_response",
        map(
            separated_pair(auth_scheme, lws, auth_params),
            |(scheme, params)| Credentials::Other(scheme.to_string(), params),
        ),
    )(input)
}

fn credentials(input: &str) -> ParserResult<&str, Credentials> {
    context("credentials", alt((digest_credentials, other_response)))(input)
}

fn authorization(input: &str) -> ParserResult<&str, Header> {
    context(
        "Authorization header",
        map(
            tuple((
                tag_no_case("Authorization"),
                hcolon,
                cut(consumed(credentials)),
            )),
            |(name, separator, (value, credentials))| {
                Header::Authorization(AuthorizationHeader::new(
                    GenericHeader::new(name, separator, value),
                    credentials,
                ))
            },
        ),
    )(input)
}

pub(crate) fn callid(input: &str) -> ParserResult<&str, CallId> {
    context(
        "callid",
        map(
            recognize(pair(word, opt(pair(tag("@"), word)))),
            CallId::new,
        ),
    )(input)
}

fn call_id(input: &str) -> ParserResult<&str, Header> {
    context(
        "Call-ID header",
        map(
            tuple((
                alt((tag_no_case("Call-ID"), tag_no_case("i"))),
                hcolon,
                cut(consumed(callid)),
            )),
            |(name, separator, (value, call_id))| {
                Header::CallId(CallIdHeader::new(
                    GenericHeader::new(name, separator, value),
                    call_id,
                ))
            },
        ),
    )(input)
}

fn info_param(input: &str) -> ParserResult<&str, CallInfoParameter> {
    context(
        "info_param",
        map(
            alt((
                map(
                    separated_pair(
                        tag_no_case("purpose"),
                        equal,
                        map(
                            alt((
                                tag_no_case("icon"),
                                tag_no_case("info"),
                                tag_no_case("card"),
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
        ),
    )(input)
}

fn info(input: &str) -> ParserResult<&str, CallInfo> {
    context(
        "info",
        map(
            tuple((
                laquot,
                absolute_uri,
                raquot,
                many0(preceded(semi, info_param)),
            )),
            |(_, uri, _, params)| CallInfo::new(uri, params),
        ),
    )(input)
}

fn call_info(input: &str) -> ParserResult<&str, Header> {
    context(
        "Call-Info header",
        map(
            tuple((
                tag_no_case("Call-Info"),
                hcolon,
                cut(consumed(separated_list1(comma, info))),
            )),
            |(name, separator, (value, infos))| {
                Header::CallInfo(CallInfoHeader::new(
                    GenericHeader::new(name, separator, value),
                    infos,
                ))
            },
        ),
    )(input)
}

fn addr_spec(input: &str) -> ParserResult<&str, Uri> {
    context(
        "addr_spec",
        alt((sip_uri, map(absolute_uri, Uri::Absolute))),
    )(input)
}

fn display_name(input: &str) -> ParserResult<&str, WrappedString> {
    context(
        "display_name",
        alt((
            quoted_string,
            map(recognize(many0(pair(token, lws))), |v| {
                WrappedString::new_not_wrapped(v.to_string().trim_end())
            }),
        )),
    )(input)
}

fn name_addr(input: &str) -> ParserResult<&str, NameAddress> {
    context(
        "name_addr",
        map(
            pair(opt(display_name), delimited(laquot, addr_spec, raquot)),
            |(display_name, uri)| NameAddress::new(uri, display_name),
        ),
    )(input)
}

fn c_p_q(input: &str) -> ParserResult<&str, ContactParameter> {
    map(
        separated_pair(tag_no_case("q"), equal, qvalue),
        |(_, value)| ContactParameter::Q(value.to_string()),
    )(input)
}

#[inline]
fn delta_seconds(input: &str) -> ParserResult<&str, u32> {
    map(recognize(many1(digit)), |digits| {
        digits.parse::<u32>().unwrap_or(u32::MAX)
    })(input)
}

fn c_p_expires(input: &str) -> ParserResult<&str, ContactParameter> {
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
fn contact_extension(input: &str) -> ParserResult<&str, GenericParameter> {
    generic_param(input)
}

fn contact_params(input: &str) -> ParserResult<&str, ContactParameter> {
    alt((c_p_q, c_p_expires, map(contact_extension, Into::into)))(input)
}

fn contact_param(input: &str) -> ParserResult<&str, Contact> {
    context(
        "contact_param",
        map(
            pair(
                alt((name_addr, map(addr_spec, |uri| NameAddress::new(uri, None)))),
                many0(preceded(semi, contact_params)),
            ),
            |(address, params)| Contact::new(address, params),
        ),
    )(input)
}

fn contact(input: &str) -> ParserResult<&str, Header> {
    context(
        "Contact header",
        map(
            tuple((
                alt((tag_no_case("Contact"), tag_no_case("m"))),
                hcolon,
                cut(consumed(alt((
                    map(star, |_| Contacts::Any),
                    map(separated_list1(comma, contact_param), Contacts::Contacts),
                )))),
            )),
            |(name, separator, (value, contacts))| {
                Header::Contact(ContactHeader::new(
                    GenericHeader::new(name, separator, value),
                    contacts,
                ))
            },
        ),
    )(input)
}

#[inline]
fn disp_extension_token(input: &str) -> ParserResult<&str, &str> {
    token(input)
}

fn disp_type(input: &str) -> ParserResult<&str, DispositionType> {
    map(
        alt((
            tag_no_case("render"),
            tag_no_case("session"),
            tag_no_case("icon"),
            tag_no_case("alert"),
            disp_extension_token,
        )),
        DispositionType::new,
    )(input)
}

#[inline]
fn other_handling(input: &str) -> ParserResult<&str, &str> {
    token(input)
}

fn handling_param(input: &str) -> ParserResult<&str, DispositionParameter> {
    map(
        separated_pair(
            tag_no_case("handling"),
            equal,
            map(
                alt((
                    tag_no_case("optional"),
                    tag_no_case("required"),
                    other_handling,
                )),
                Handling::new,
            ),
        ),
        |(_, value)| DispositionParameter::Handling(value),
    )(input)
}

fn disp_param(input: &str) -> ParserResult<&str, DispositionParameter> {
    alt((handling_param, map(generic_param, Into::into)))(input)
}

fn content_disposition(input: &str) -> ParserResult<&str, Header> {
    context(
        "Content-Disposition header",
        map(
            tuple((
                tag_no_case("Content-Disposition"),
                hcolon,
                cut(consumed(pair(disp_type, many0(preceded(semi, disp_param))))),
            )),
            |(name, separator, (value, (r#type, params)))| {
                Header::ContentDisposition(ContentDispositionHeader::new(
                    GenericHeader::new(name, separator, value),
                    r#type,
                    params,
                ))
            },
        ),
    )(input)
}

fn content_encoding(input: &str) -> ParserResult<&str, Header> {
    context(
        "Content-Encoding header",
        map(
            tuple((
                alt((tag_no_case("Content-Encoding"), tag("e"))),
                hcolon,
                cut(consumed(separated_list1(comma, content_coding))),
            )),
            |(name, separator, (value, encodings))| {
                Header::ContentEncoding(ContentEncodingHeader::new(
                    GenericHeader::new(name, separator, value),
                    encodings,
                ))
            },
        ),
    )(input)
}

#[inline]
fn primary_tag(input: &str) -> ParserResult<&str, &str> {
    recognize(many_m_n(1, 8, alpha))(input)
}

#[inline]
fn subtag(input: &str) -> ParserResult<&str, &str> {
    primary_tag(input)
}

pub(crate) fn language_tag(input: &str) -> ParserResult<&str, ContentLanguage> {
    map(
        recognize(pair(primary_tag, many0(preceded(tag("-"), subtag)))),
        ContentLanguage::new,
    )(input)
}

fn content_language(input: &str) -> ParserResult<&str, Header> {
    context(
        "Content-Language header",
        map(
            tuple((
                tag_no_case("Content-Language"),
                hcolon,
                cut(consumed(separated_list1(comma, language_tag))),
            )),
            |(name, separator, (value, languages))| {
                Header::ContentLanguage(ContentLanguageHeader::new(
                    GenericHeader::new(name, separator, value),
                    languages,
                ))
            },
        ),
    )(input)
}

fn content_length(input: &str) -> ParserResult<&str, Header> {
    context(
        "Content-Length header",
        map(
            tuple((
                alt((tag_no_case("Content-Length"), tag_no_case("l"))),
                hcolon,
                cut(consumed(map(recognize(many1(digit)), |l| {
                    l.parse::<u32>().unwrap()
                }))),
            )),
            |(name, separator, (value, content_length))| {
                Header::ContentLength(ContentLengthHeader::new(
                    GenericHeader::new(name, separator, value),
                    content_length,
                ))
            },
        ),
    )(input)
}

#[inline]
fn m_attribute(input: &str) -> ParserResult<&str, &str> {
    token(input)
}

fn m_value(input: &str) -> ParserResult<&str, WrappedString> {
    context(
        "m_value",
        alt((map(token, WrappedString::new_not_wrapped), quoted_string)),
    )(input)
}

fn m_parameter(input: &str) -> ParserResult<&str, MediaParameter> {
    context(
        "m_parameter",
        map(
            separated_pair(m_attribute, equal, m_value),
            |(key, value)| MediaParameter::new(key, value),
        ),
    )(input)
}

fn media_type(input: &str) -> ParserResult<&str, MediaType> {
    context(
        "media_type",
        map(
            tuple((m_type, slash, m_subtype, many0(preceded(semi, m_parameter)))),
            |(r#type, _, subtype, parameters)| {
                MediaType::new(MediaRange::new(r#type, subtype), parameters)
            },
        ),
    )(input)
}

fn content_type(input: &str) -> ParserResult<&str, Header> {
    context(
        "Content-Type header",
        map(
            tuple((
                alt((tag_no_case("Content-Type"), tag_no_case("c"))),
                hcolon,
                cut(consumed(media_type)),
            )),
            |(name, separator, (value, media_type))| {
                Header::ContentType(ContentTypeHeader::new(
                    GenericHeader::new(name, separator, value),
                    media_type,
                ))
            },
        ),
    )(input)
}

fn cseq(input: &str) -> ParserResult<&str, Header> {
    context(
        "CSeq header",
        map(
            tuple((
                tag_no_case("CSeq"),
                hcolon,
                cut(consumed(separated_pair(
                    map(recognize(many1(digit)), |cseq| cseq.parse::<u32>().unwrap()),
                    lws,
                    method,
                ))),
            )),
            |(name, separator, (value, (cseq, method)))| {
                Header::CSeq(CSeqHeader::new(
                    GenericHeader::new(name, separator, value),
                    cseq,
                    method,
                ))
            },
        ),
    )(input)
}

fn wkday(input: &str) -> ParserResult<&str, &str> {
    context(
        "wkday",
        alt((
            tag("Mon"),
            tag("Tue"),
            tag("Wed"),
            tag("Thu"),
            tag("Fri"),
            tag("Sat"),
            tag("Sun"),
        )),
    )(input)
}

fn month(input: &str) -> ParserResult<&str, &str> {
    context(
        "month",
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
    )(input)
}

fn date1(input: &str) -> ParserResult<&str, &str> {
    context(
        "date1",
        recognize(tuple((count(digit, 2), sp, month, sp, count(digit, 4)))),
    )(input)
}

fn time(input: &str) -> ParserResult<&str, &str> {
    context(
        "time",
        recognize(tuple((
            count(digit, 2),
            tag(":"),
            count(digit, 2),
            tag(":"),
            count(digit, 2),
        ))),
    )(input)
}

fn rfc1123_date(input: &str) -> ParserResult<&str, DateTime<Utc>> {
    let result = recognize(tuple((
        wkday,
        tag(","),
        sp,
        date1,
        sp,
        time,
        sp,
        tag("GMT"),
    )))(input);
    match result {
        Err(e) => Err(e),
        Ok((rest, date)) => {
            let result = DateTime::parse_from_rfc2822(date);
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
fn sip_date(input: &str) -> ParserResult<&str, DateTime<Utc>> {
    rfc1123_date(input)
}

fn date(input: &str) -> ParserResult<&str, Header> {
    context(
        "Date header",
        map(
            tuple((tag_no_case("Date"), hcolon, cut(consumed(sip_date)))),
            |(name, separator, (value, date))| {
                Header::Date(DateHeader::new(
                    GenericHeader::new(name, separator, value),
                    date,
                ))
            },
        ),
    )(input)
}

fn error_uri(input: &str) -> ParserResult<&str, ErrorUri> {
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

fn error_info(input: &str) -> ParserResult<&str, Header> {
    context(
        "Error-Info header",
        map(
            tuple((
                tag_no_case("Error-Info"),
                hcolon,
                cut(consumed(separated_list1(comma, error_uri))),
            )),
            |(name, separator, (value, uris))| {
                Header::ErrorInfo(ErrorInfoHeader::new(
                    GenericHeader::new(name, separator, value),
                    uris,
                ))
            },
        ),
    )(input)
}

fn expires(input: &str) -> ParserResult<&str, Header> {
    context(
        "Expires header",
        map(
            tuple((tag_no_case("Expires"), hcolon, cut(consumed(delta_seconds)))),
            |(name, separator, (value, expires))| {
                Header::Expires(ExpiresHeader::new(
                    GenericHeader::new(name, separator, value),
                    expires,
                ))
            },
        ),
    )(input)
}

fn tag_param(input: &str) -> ParserResult<&str, GenericParameter> {
    context(
        "tag_param",
        map(
            separated_pair(tag_no_case("tag"), equal, token),
            |(key, value)| GenericParameter::new(key, Some(value)),
        ),
    )(input)
}

fn from_param(input: &str) -> ParserResult<&str, FromParameter> {
    context(
        "from_param",
        map(alt((tag_param, generic_param)), Into::into),
    )(input)
}

fn from_spec(input: &str) -> ParserResult<&str, (NameAddress, Vec<FromParameter>)> {
    context(
        "from_spec",
        pair(
            alt((map(addr_spec, |uri| NameAddress::new(uri, None)), name_addr)),
            many0(preceded(semi, from_param)),
        ),
    )(input)
}

fn from(input: &str) -> ParserResult<&str, Header> {
    context(
        "From header",
        map(
            tuple((
                alt((tag_no_case("From"), tag_no_case("f"))),
                hcolon,
                cut(consumed(from_spec)),
            )),
            |(name, separator, (value, (address, parameters)))| {
                Header::From(FromHeader::new(
                    GenericHeader::new(name, separator, value),
                    address,
                    parameters,
                ))
            },
        ),
    )(input)
}

fn in_reply_to(input: &str) -> ParserResult<&str, Header> {
    context(
        "In-Reply-To header",
        map(
            tuple((
                tag_no_case("In-Reply-To"),
                hcolon,
                cut(consumed(separated_list1(comma, callid))),
            )),
            |(name, separator, (value, call_ids))| {
                Header::InReplyTo(InReplyToHeader::new(
                    GenericHeader::new(name, separator, value),
                    call_ids,
                ))
            },
        ),
    )(input)
}

fn max_forwards(input: &str) -> ParserResult<&str, Header> {
    context(
        "Max-Forwards header",
        map(
            tuple((
                tag_no_case("Max-Forwards"),
                hcolon,
                cut(consumed(map(recognize(many1(digit)), |value| {
                    value.parse::<u8>().unwrap_or(u8::MAX)
                }))),
            )),
            |(name, separator, (value, max_forwards))| {
                Header::MaxForwards(MaxForwardsHeader::new(
                    GenericHeader::new(name, separator, value),
                    max_forwards,
                ))
            },
        ),
    )(input)
}

fn mime_version(input: &str) -> ParserResult<&str, Header> {
    context(
        "MIME-Version header",
        map(
            tuple((
                tag_no_case("MIME-Version"),
                hcolon,
                cut(consumed(recognize(tuple((
                    many1(digit),
                    tag("."),
                    many1(digit),
                ))))),
            )),
            |(name, separator, (value, version))| {
                Header::MimeVersion(MimeVersionHeader::new(
                    GenericHeader::new(name, separator, value),
                    version,
                ))
            },
        ),
    )(input)
}

fn min_expires(input: &str) -> ParserResult<&str, Header> {
    context(
        "Min-Expires header",
        map(
            tuple((
                tag_no_case("Min-Expires"),
                hcolon,
                cut(consumed(delta_seconds)),
            )),
            |(name, separator, (value, min_expires))| {
                Header::MinExpires(MinExpiresHeader::new(
                    GenericHeader::new(name, separator, value),
                    min_expires,
                ))
            },
        ),
    )(input)
}

fn organization(input: &str) -> ParserResult<&str, Header> {
    context(
        "Organization header",
        map(
            tuple((
                tag_no_case("Organization"),
                hcolon,
                cut(consumed(opt(text_utf8_trim))),
            )),
            |(name, separator, (value, organization))| {
                Header::Organization(OrganizationHeader::new(
                    GenericHeader::new(name, separator, value),
                    organization.unwrap_or_default(),
                ))
            },
        ),
    )(input)
}

#[inline]
fn other_priority(input: &str) -> ParserResult<&str, &str> {
    token(input)
}

fn priority_value(input: &str) -> ParserResult<&str, Priority> {
    context(
        "priority_value",
        map(
            alt((
                tag_no_case("emergency"),
                tag_no_case("urgent"),
                tag_no_case("normal"),
                tag_no_case("non-urgent"),
                other_priority,
            )),
            Priority::new,
        ),
    )(input)
}

fn priority(input: &str) -> ParserResult<&str, Header> {
    context(
        "Priority header",
        map(
            tuple((
                tag_no_case("Priority"),
                hcolon,
                cut(consumed(priority_value)),
            )),
            |(name, separator, (value, priority))| {
                Header::Priority(PriorityHeader::new(
                    GenericHeader::new(name, separator, value),
                    priority,
                ))
            },
        ),
    )(input)
}

fn other_challenge(input: &str) -> ParserResult<&str, Challenge> {
    context(
        "other_challenge",
        map(
            separated_pair(auth_scheme, lws, separated_list1(comma, auth_param)),
            |(scheme, auth_params)| Challenge::Other(scheme.to_string(), auth_params.into()),
        ),
    )(input)
}

fn segment(input: &str) -> ParserResult<&str, &str> {
    context(
        "segment",
        recognize(pair(many0(pchar), many0(preceded(tag(";"), param)))),
    )(input)
}

fn path_segments(input: &str) -> ParserResult<&str, &str> {
    context(
        "path_segments",
        recognize(pair(segment, many0(preceded(tag("/"), segment)))),
    )(input)
}

fn abs_path(input: &str) -> ParserResult<&str, &str> {
    context("abs_path", recognize(pair(tag("/"), path_segments)))(input)
}

fn uri(input: &str) -> ParserResult<&str, DomainUri> {
    context(
        "uri",
        alt((
            map(request_uri, DomainUri::Uri),
            map(abs_path, |path| DomainUri::AbsPath(path.to_string())),
        )),
    )(input)
}

fn domain_value(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "domain_value",
        delimited(
            ldquot,
            map(separated_list1(many1(sp), uri), |uris| {
                AuthParameter::Domain(uris.into())
            }),
            rdquot,
        ),
    )(input)
}

fn domain(input: &str) -> ParserResult<&str, AuthParameter> {
    map(
        tuple((tag_no_case("domain"), equal, cut(domain_value))),
        |(_, _, domain)| domain,
    )(input)
}

fn stale(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "stale",
        map(
            separated_pair(
                tag_no_case("stale"),
                equal,
                cut(map(
                    consumed(alt((
                        value(true, tag_no_case("true")),
                        value(false, tag_no_case("false")),
                    ))),
                    |(s, v)| AuthParameter::Stale(Stale::new(s, v)),
                )),
            ),
            |(_, stale)| stale,
        ),
    )(input)
}

fn qop_options(input: &str) -> ParserResult<&str, AuthParameter> {
    context(
        "qop_options",
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
        ),
    )(input)
}

fn digest_cln(input: &str) -> ParserResult<&str, AuthParameter> {
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

fn challenge(input: &str) -> ParserResult<&str, Challenge> {
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

fn proxy_authenticate(input: &str) -> ParserResult<&str, Header> {
    context(
        "Proxy-Authenticate header",
        map(
            tuple((
                tag_no_case("Proxy-Authenticate"),
                hcolon,
                cut(consumed(challenge)),
            )),
            |(name, separator, (value, challenge))| {
                Header::ProxyAuthenticate(ProxyAuthenticateHeader::new(
                    GenericHeader::new(name, separator, value),
                    challenge,
                ))
            },
        ),
    )(input)
}

fn proxy_authorization(input: &str) -> ParserResult<&str, Header> {
    context(
        "Proxy-Authorization header",
        map(
            tuple((
                tag_no_case("Proxy-Authorization"),
                hcolon,
                cut(consumed(credentials)),
            )),
            |(name, separator, (value, credentials))| {
                Header::ProxyAuthorization(ProxyAuthorizationHeader::new(
                    GenericHeader::new(name, separator, value),
                    credentials,
                ))
            },
        ),
    )(input)
}

pub(crate) fn option_tag(input: &str) -> ParserResult<&str, OptionTag> {
    context("option_tag", map(token, OptionTag::new))(input)
}

fn proxy_require(input: &str) -> ParserResult<&str, Header> {
    context(
        "Proxy-Require header",
        map(
            tuple((
                tag_no_case("Proxy-Require"),
                hcolon,
                cut(consumed(separated_list1(comma, option_tag))),
            )),
            |(name, separator, (value, tags))| {
                Header::ProxyRequire(ProxyRequireHeader::new(
                    GenericHeader::new(name, separator, value),
                    tags,
                ))
            },
        ),
    )(input)
}

#[inline]
fn route_param(input: &str) -> ParserResult<&str, GenericParameter> {
    generic_param(input)
}

fn route(input: &str) -> ParserResult<&str, Route> {
    context(
        "route",
        map(
            pair(name_addr, many0(route_param)),
            |(name_addr, params)| Route::new(name_addr, params),
        ),
    )(input)
}

fn record_route(input: &str) -> ParserResult<&str, Header> {
    context(
        "Record-Route header",
        map(
            tuple((
                tag_no_case("Record-Route"),
                hcolon,
                cut(consumed(separated_list1(comma, route))),
            )),
            |(name, separator, (value, routes))| {
                Header::RecordRoute(RecordRouteHeader::new(
                    GenericHeader::new(name, separator, value),
                    routes,
                ))
            },
        ),
    )(input)
}

#[inline]
fn header_name(input: &str) -> ParserResult<&str, &str> {
    token(input)
}

fn header_value(input: &str) -> ParserResult<&str, &str> {
    context(
        "header_value",
        recognize(many0(alt((recognize(text_utf8char), lws)))),
    )(input)
}

fn extension_header(input: &str) -> ParserResult<&str, Header> {
    map(
        tuple((header_name, hcolon, header_value)),
        |(name, separator, value)| {
            Header::ExtensionHeader(GenericHeader::new(name, separator, value))
        },
    )(input)
}

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
                extension_header,
            )),
        )),
    )(input)
}
