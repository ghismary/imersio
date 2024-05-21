use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt, recognize, verify},
    error::context,
    multi::{count, many0, many_m_n},
    sequence::{delimited, pair, preceded, separated_pair},
};

use crate::{
    method::parser::method,
    parser::{
        alpha, comma, digit, equal, hcolon, laquot, ldquot, lhex, lws, quoted_string, raquot,
        rdquot, semi, slash, token, word, ParserResult,
    },
    uri::parser::{absolute_uri, host},
};

use super::{
    accept_encoding_header::{AcceptEncodingHeader, Encoding},
    accept_header::{AcceptHeader, AcceptParameter, AcceptRange, MediaRange},
    accept_language_header::{AcceptLanguageHeader, Language},
    alert_info_header::{AlertInfoHeader, AlertParam},
    allow_header::AllowHeader,
    authentication_info_header::{AInfo, AuthenticationInfoHeader},
    authorization_header::{AuthParam, AuthorizationHeader, Credentials},
    call_id_header::CallIdHeader,
    Header,
};

fn discrete_type(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((
        tag("text"),
        tag("image"),
        tag("audio"),
        tag("video"),
        tag("application"),
    ))(input)
    .map(|(rest, value)| (rest, String::from_utf8_lossy(value)))
}

fn composite_type(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((tag("message"), tag("multipart")))(input)
        .map(|(rest, value)| (rest, String::from_utf8_lossy(value)))
}

#[inline]
fn ietf_token(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn x_token(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    recognize(pair(tag("x-"), token))(input)
        .map(|(rest, value)| (rest, String::from_utf8_lossy(value)))
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
    alt((
        separated_pair(
            map(tag("*"), String::from_utf8_lossy),
            tag("/"),
            map(tag("*"), String::from_utf8_lossy),
        ),
        separated_pair(m_type, slash, map(tag("*"), String::from_utf8_lossy)),
        separated_pair(m_type, slash, m_subtype),
    ))(input)
    .map(|(rest, (r#type, subtype))| {
        (
            rest,
            MediaRange::new(r#type.into_owned(), subtype.into_owned()),
        )
    })
}

fn qvalue(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    recognize(alt((
        pair(tag("0"), opt(pair(tag("."), many_m_n(0, 3, digit)))),
        pair(tag("1"), opt(pair(tag("."), many_m_n(0, 3, tag("0"))))),
    )))(input)
    .map(|(rest, qvalue)| (rest, String::from_utf8_lossy(qvalue)))
}

fn q_param(input: &[u8]) -> ParserResult<&[u8], AcceptParameter> {
    separated_pair(tag("q"), equal, qvalue)(input).map(|(rest, (key, value))| {
        (
            rest,
            AcceptParameter::new(
                String::from_utf8_lossy(key).into_owned(),
                Some(value.into_owned()),
            ),
        )
    })
}

fn gen_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((token, host, quoted_string))(input)
}

fn generic_param(input: &[u8]) -> ParserResult<&[u8], AcceptParameter> {
    pair(token, opt(preceded(equal, gen_value)))(input).map(|(rest, (key, value))| {
        (
            rest,
            AcceptParameter::new(key.into_owned(), value.map(|v| v.into_owned())),
        )
    })
}

fn accept_param(input: &[u8]) -> ParserResult<&[u8], AcceptParameter> {
    context("accept_param", alt((q_param, generic_param)))(input)
}

fn accept_range(input: &[u8]) -> ParserResult<&[u8], AcceptRange> {
    pair(media_range, many0(preceded(semi, accept_param)))(input).map(
        |(rest, (media_range, accept_parameters))| {
            (rest, AcceptRange::new(media_range, accept_parameters))
        },
    )
}

fn accept(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "accept",
        separated_pair(
            tag("Accept"),
            hcolon,
            opt(pair(accept_range, many0(preceded(comma, accept_range)))),
        ),
    )(input)
    .map(|(rest, (_, ranges))| {
        (
            rest,
            Header::Accept(AcceptHeader::new(match ranges {
                Some((first_range, mut other_ranges)) => {
                    other_ranges.insert(0, first_range);
                    other_ranges
                }
                None => vec![],
            })),
        )
    })
}

#[inline]
fn content_coding(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn codings(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((content_coding, map(tag("*"), String::from_utf8_lossy)))(input)
}

fn encoding(input: &[u8]) -> ParserResult<&[u8], Encoding> {
    context(
        "encoding",
        pair(codings, many0(preceded(semi, accept_param))),
    )(input)
    .map(|(rest, (codings, params))| (rest, Encoding::new(codings.into_owned(), params)))
}

fn accept_encoding(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "accept_encoding",
        separated_pair(
            tag("Accept-Encoding"),
            hcolon,
            opt(pair(encoding, many0(preceded(comma, encoding)))),
        ),
    )(input)
    .map(|(rest, (_, encodings))| {
        (
            rest,
            Header::AcceptEncoding(AcceptEncodingHeader::new(match encodings {
                Some((first_encoding, mut other_encodings)) => {
                    other_encodings.insert(0, first_encoding);
                    other_encodings
                }
                None => vec![],
            })),
        )
    })
}

fn language_range(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
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
    .map(|(rest, language)| (rest, String::from_utf8_lossy(language)))
}

fn language(input: &[u8]) -> ParserResult<&[u8], Language> {
    context(
        "language",
        pair(language_range, many0(preceded(semi, accept_param))),
    )(input)
    .map(|(rest, (language, params))| (rest, Language::new(language.into_owned(), params)))
}

fn accept_language(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "accept_language",
        separated_pair(
            tag("Accept-Language"),
            hcolon,
            opt(pair(language, many0(preceded(comma, language)))),
        ),
    )(input)
    .map(|(rest, (_, languages))| {
        (
            rest,
            Header::AcceptLanguage(AcceptLanguageHeader::new(match languages {
                Some((first_language, mut other_languages)) => {
                    other_languages.insert(0, first_language);
                    other_languages
                }
                None => vec![],
            })),
        )
    })
}

fn alert_param(input: &[u8]) -> ParserResult<&[u8], AlertParam> {
    context(
        "alert_param",
        pair(
            delimited(laquot, absolute_uri, raquot),
            many0(preceded(semi, generic_param)),
        ),
    )(input)
    .map(|(rest, (uri, params))| (rest, AlertParam::new(uri, params)))
}

fn alert_info(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "alert_info",
        separated_pair(
            tag("Alert-Info"),
            hcolon,
            pair(alert_param, many0(preceded(comma, alert_param))),
        ),
    )(input)
    .map(|(rest, (_, (first_alert, mut other_alerts)))| {
        other_alerts.insert(0, first_alert);
        (rest, Header::AlertInfo(AlertInfoHeader::new(other_alerts)))
    })
}

fn allow(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "allow",
        separated_pair(
            tag("Allow"),
            hcolon,
            opt(pair(method, many0(preceded(comma, method)))),
        ),
    )(input)
    .map(|(rest, (_, methods))| {
        (
            rest,
            Header::Allow(AllowHeader::new(match methods {
                Some((first_method, mut other_methods)) => {
                    other_methods.insert(0, first_method);
                    other_methods
                }
                None => vec![],
            })),
        )
    })
}

#[inline]
fn nonce_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    quoted_string(input)
}

fn nextnonce(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context(
        "nextnonce",
        separated_pair(tag("nextnonce"), equal, nonce_value),
    )(input)
    .map(|(rest, (_, value))| (rest, AInfo::NextNonce(value.into_owned())))
}

fn qop_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((
        map(tag("auth-int"), String::from_utf8_lossy),
        map(tag("auth"), String::from_utf8_lossy),
        token,
    ))(input)
}

fn message_qop(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context("message_qop", separated_pair(tag("qop"), equal, qop_value))(input)
        .map(|(rest, (_, value))| (rest, AInfo::MessageQop(value.into_owned())))
}

fn response_digest(input: &[u8]) -> ParserResult<&[u8], String> {
    delimited(ldquot, many0(lhex), rdquot)(input).map(|(rest, digits)| {
        (
            rest,
            digits
                .iter()
                .map(|digit| String::from_utf8_lossy(digit).into_owned())
                .collect::<Vec<String>>()
                .join(""),
        )
    })
}

fn response_auth(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context(
        "response_auth",
        separated_pair(tag("rspauth"), equal, response_digest),
    )(input)
    .map(|(rest, (_, value))| (rest, AInfo::ResponseAuth(value)))
}

#[inline]
fn cnonce_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    nonce_value(input)
}

fn cnonce(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context("cnonce", separated_pair(tag("cnonce"), equal, cnonce_value))(input)
        .map(|(rest, (_, value))| (rest, AInfo::CNonce(value.into_owned())))
}

fn nc_value(input: &[u8]) -> ParserResult<&[u8], String> {
    count(lhex, 8)(input).map(|(rest, digits)| {
        (
            rest,
            digits
                .iter()
                .map(|digit| String::from_utf8_lossy(digit).into_owned())
                .collect::<Vec<String>>()
                .join(""),
        )
    })
}

fn nonce_count(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context("nonce_count", separated_pair(tag("nc"), equal, nc_value))(input)
        .map(|(rest, (_, value))| (rest, AInfo::NonceCount(value)))
}

fn ainfo(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    alt((nextnonce, message_qop, response_auth, cnonce, nonce_count))(input)
}

fn authentication_info(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "authentication_info",
        separated_pair(
            tag("Authentication-Info"),
            hcolon,
            pair(ainfo, many0(preceded(comma, ainfo))),
        ),
    )(input)
    .map(|(rest, (_, (first_ainfo, mut other_ainfos)))| {
        other_ainfos.insert(0, first_ainfo);
        (
            rest,
            Header::AuthenticationInfo(AuthenticationInfoHeader::new(other_ainfos)),
        )
    })
}

#[inline]
fn username_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    quoted_string(input)
}

fn username(input: &[u8]) -> ParserResult<&[u8], AuthParam> {
    separated_pair(tag("username"), equal, username_value)(input)
        .map(|(rest, (_, value))| (rest, AuthParam::Username(value.into_owned())))
}

#[inline]
fn realm_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    quoted_string(input)
}

fn realm(input: &[u8]) -> ParserResult<&[u8], AuthParam> {
    separated_pair(tag("realm"), equal, realm_value)(input)
        .map(|(rest, (_, value))| (rest, AuthParam::Realm(value.into_owned())))
}

fn nonce(input: &[u8]) -> ParserResult<&[u8], AuthParam> {
    separated_pair(tag("nonce"), equal, nonce_value)(input)
        .map(|(rest, (_, value))| (rest, AuthParam::Nonce(value.into_owned())))
}

fn digest_uri_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    // TODO: Handel rquest-uri
    // delimited(ldquot, rquest_uri, rdquot)(input)
    quoted_string(input)
}

fn digest_uri(input: &[u8]) -> ParserResult<&[u8], AuthParam> {
    separated_pair(tag("uri"), equal, digest_uri_value)(input)
        .map(|(rest, (_, value))| (rest, AuthParam::DigestUri(value.into_owned())))
}

fn request_digest(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    delimited(ldquot, recognize(many_m_n(32, 32, lhex)), rdquot)(input)
        .map(|(rest, value)| (rest, String::from_utf8_lossy(value)))
}

fn dresponse(input: &[u8]) -> ParserResult<&[u8], AuthParam> {
    separated_pair(tag("response"), equal, request_digest)(input)
        .map(|(rest, (_, value))| (rest, AuthParam::DResponse(value.into_owned())))
}

fn algorithm(input: &[u8]) -> ParserResult<&[u8], AuthParam> {
    separated_pair(
        tag("algorithm"),
        equal,
        alt((
            map(tag("MD5"), String::from_utf8_lossy),
            map(tag("MD5-sess"), String::from_utf8_lossy),
            token,
        )),
    )(input)
    .map(|(rest, (_, value))| (rest, AuthParam::Algorithm(Cow::Owned(value.into_owned()))))
}

fn opaque(input: &[u8]) -> ParserResult<&[u8], AuthParam> {
    separated_pair(tag("opaque"), equal, quoted_string)(input)
        .map(|(rest, (_, value))| (rest, AuthParam::Opaque(value.into_owned())))
}

#[inline]
fn auth_param_name(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn auth_param(input: &[u8]) -> ParserResult<&[u8], AuthParam> {
    separated_pair(auth_param_name, equal, alt((token, quoted_string)))(input)
        .map(|(rest, (key, value))| (rest, AuthParam::Other(key.to_string(), value.to_string())))
}

fn dig_resp(input: &[u8]) -> ParserResult<&[u8], AuthParam> {
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
        verify(auth_param, |param| {
            ![
                "username",
                "realm",
                "nonce",
                "uri",
                "response",
                "algorithm",
                "cnonce",
                "opaque",
                "qop",
                "nc",
            ]
            .contains(&param.key())
        }),
    ))(input)
}

fn digest_response(input: &[u8]) -> ParserResult<&[u8], Vec<AuthParam>> {
    pair(dig_resp, many0(preceded(comma, dig_resp)))(input).map(
        |(rest, (first_param, mut other_params))| {
            other_params.insert(0, first_param);
            (rest, other_params)
        },
    )
}

#[inline]
fn auth_scheme(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn auth_params(input: &[u8]) -> ParserResult<&[u8], Vec<AuthParam>> {
    pair(auth_param, many0(preceded(comma, auth_param)))(input).map(
        |(rest, (first_param, mut other_params))| {
            other_params.insert(0, first_param);
            (rest, other_params)
        },
    )
}

fn digest_credentials(input: &[u8]) -> ParserResult<&[u8], Credentials> {
    separated_pair(
        map(tag("Digest"), String::from_utf8_lossy),
        lws,
        digest_response,
    )(input)
    .map(|(rest, (_, params))| (rest, Credentials::Digest(params)))
}

fn other_response(input: &[u8]) -> ParserResult<&[u8], Credentials> {
    separated_pair(
        verify(auth_scheme, |s: &Cow<'_, str>| s != "Digest"),
        lws,
        auth_params,
    )(input)
    .map(|(rest, (scheme, params))| (rest, Credentials::Other(scheme.to_string(), params)))
}

fn credentials(input: &[u8]) -> ParserResult<&[u8], Credentials> {
    alt((digest_credentials, other_response))(input)
}

fn authorization(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "authorization",
        separated_pair(tag("Authorization"), hcolon, credentials),
    )(input)
    .map(|(rest, (_, credentials))| {
        (
            rest,
            Header::Authorization(AuthorizationHeader::new(credentials)),
        )
    })
}

fn callid(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    recognize(pair(word, opt(pair(tag("@"), word))))(input)
        .map(|(rest, value)| (rest, String::from_utf8_lossy(value)))
}

fn call_id(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "call_id",
        separated_pair(alt((tag("Call-ID"), tag("i"))), hcolon, callid),
    )(input)
    .map(|(rest, (_, call_id))| (rest, Header::CallId(CallIdHeader::new(call_id.to_string()))))
}

pub(super) fn message_header(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "message_header",
        alt((
            accept,
            accept_encoding,
            accept_language,
            alert_info,
            allow,
            authentication_info,
            authorization,
            call_id,
        )),
    )(input)
}
