use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt, recognize, verify},
    error::context,
    multi::{count, many0, many1, many_m_n},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};

use crate::{
    common::{AcceptParameter, Algorithm, MessageQop, NameAddress},
    method::parser::method,
    parser::{
        alpha, comma, digit, equal, hcolon, laquot, ldquot, lhex, lws, quoted_string, raquot,
        rdquot, semi, slash, star, token, word, ParserResult,
    },
    uri::parser::{absolute_uri, host, sip_uri, sips_uri},
    GenericParameter, Uri,
};

use super::{
    accept_encoding_header::{AcceptEncodingHeader, Encoding},
    accept_header::{AcceptHeader, AcceptRange, MediaRange},
    accept_language_header::{AcceptLanguageHeader, Language},
    alert_info_header::{AlertInfoHeader, AlertParameter},
    allow_header::AllowHeader,
    authentication_info_header::{AInfo, AuthenticationInfoHeader},
    authorization_header::{AuthParameter, AuthorizationHeader, Credentials},
    call_id_header::CallIdHeader,
    call_info_header::{CallInfo, CallInfoHeader, CallInfoParameter},
    contact_header::{Contact, ContactHeader, ContactParameter},
    Header,
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
        |(r#type, subtype)| MediaRange::new(r#type.into_owned(), subtype.into_owned()),
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
        AcceptParameter::new(
            String::from_utf8_lossy(key).into_owned(),
            Some(value.into_owned()),
        )
    })(input)
}

fn gen_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((token, host, quoted_string))(input)
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
            separated_pair(
                tag("Accept"),
                hcolon,
                opt(pair(accept_range, many0(preceded(comma, accept_range)))),
            ),
            |(_, ranges)| {
                Header::Accept(AcceptHeader::new(match ranges {
                    Some((first_range, mut other_ranges)) => {
                        other_ranges.insert(0, first_range);
                        other_ranges
                    }
                    None => vec![],
                }))
            },
        ),
    )(input)
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
        map(
            pair(codings, many0(preceded(semi, accept_param))),
            |(codings, params)| Encoding::new(codings.into_owned(), params),
        ),
    )(input)
}

fn accept_encoding(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "accept_encoding",
        map(
            separated_pair(
                tag("Accept-Encoding"),
                hcolon,
                opt(pair(encoding, many0(preceded(comma, encoding)))),
            ),
            |(_, encodings)| {
                Header::AcceptEncoding(AcceptEncodingHeader::new(match encodings {
                    Some((first_encoding, mut other_encodings)) => {
                        other_encodings.insert(0, first_encoding);
                        other_encodings
                    }
                    None => vec![],
                }))
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

fn language(input: &[u8]) -> ParserResult<&[u8], Language> {
    context(
        "language",
        map(
            pair(language_range, many0(preceded(semi, accept_param))),
            |(language, params)| Language::new(language.into_owned(), params),
        ),
    )(input)
}

fn accept_language(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "accept_language",
        map(
            separated_pair(
                tag("Accept-Language"),
                hcolon,
                opt(pair(language, many0(preceded(comma, language)))),
            ),
            |(_, languages)| {
                Header::AcceptLanguage(AcceptLanguageHeader::new(match languages {
                    Some((first_language, mut other_languages)) => {
                        other_languages.insert(0, first_language);
                        other_languages
                    }
                    None => vec![],
                }))
            },
        ),
    )(input)
}

fn alert_param(input: &[u8]) -> ParserResult<&[u8], AlertParameter> {
    context(
        "alert_param",
        map(
            pair(
                delimited(laquot, absolute_uri, raquot),
                many0(preceded(semi, map(generic_param, Into::into))),
            ),
            |(uri, params)| AlertParameter::new(uri, params),
        ),
    )(input)
}

fn alert_info(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "alert_info",
        map(
            separated_pair(
                tag("Alert-Info"),
                hcolon,
                pair(alert_param, many0(preceded(comma, alert_param))),
            ),
            |(_, (first_alert, mut other_alerts))| {
                other_alerts.insert(0, first_alert);
                Header::AlertInfo(AlertInfoHeader::new(other_alerts))
            },
        ),
    )(input)
}

fn allow(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "allow",
        map(
            separated_pair(
                tag("Allow"),
                hcolon,
                opt(pair(method, many0(preceded(comma, method)))),
            ),
            |(_, methods)| {
                Header::Allow(AllowHeader::new(match methods {
                    Some((first_method, mut other_methods)) => {
                        other_methods.insert(0, first_method);
                        other_methods
                    }
                    None => vec![],
                }))
            },
        ),
    )(input)
}

#[inline]
fn nonce_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    quoted_string(input)
}

fn nextnonce(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context(
        "nextnonce",
        map(
            separated_pair(tag("nextnonce"), equal, nonce_value),
            |(_, value)| AInfo::NextNonce(value.into_owned()),
        ),
    )(input)
}

fn qop_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((
        map(tag("auth-int"), String::from_utf8_lossy),
        map(tag("auth"), String::from_utf8_lossy),
        token,
    ))(input)
}

fn message_qop(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context(
        "message_qop",
        map(
            separated_pair(tag("qop"), equal, qop_value),
            |(_, value)| AInfo::MessageQop(MessageQop::new(value.into_owned())),
        ),
    )(input)
}

fn response_digest(input: &[u8]) -> ParserResult<&[u8], String> {
    map(delimited(ldquot, many0(lhex), rdquot), |digits| {
        digits
            .iter()
            .map(|digit| String::from_utf8_lossy(digit).into_owned())
            .collect::<Vec<String>>()
            .join("")
    })(input)
}

fn response_auth(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context(
        "response_auth",
        map(
            separated_pair(tag("rspauth"), equal, response_digest),
            |(_, value)| AInfo::ResponseAuth(value),
        ),
    )(input)
}

#[inline]
fn cnonce_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    nonce_value(input)
}

fn cnonce(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context(
        "cnonce",
        map(
            separated_pair(tag("cnonce"), equal, cnonce_value),
            |(_, value)| AInfo::CNonce(value.into_owned()),
        ),
    )(input)
}

fn nc_value(input: &[u8]) -> ParserResult<&[u8], String> {
    map(count(lhex, 8), |digits| {
        digits
            .iter()
            .map(|digit| String::from_utf8_lossy(digit).into_owned())
            .collect::<Vec<String>>()
            .join("")
    })(input)
}

fn nonce_count(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    context(
        "nonce_count",
        map(separated_pair(tag("nc"), equal, nc_value), |(_, value)| {
            AInfo::NonceCount(value)
        }),
    )(input)
}

fn ainfo(input: &[u8]) -> ParserResult<&[u8], AInfo> {
    alt((nextnonce, message_qop, response_auth, cnonce, nonce_count))(input)
}

fn authentication_info(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "authentication_info",
        map(
            separated_pair(
                tag("Authentication-Info"),
                hcolon,
                pair(ainfo, many0(preceded(comma, ainfo))),
            ),
            |(_, (first_ainfo, mut other_ainfos))| {
                other_ainfos.insert(0, first_ainfo);
                Header::AuthenticationInfo(AuthenticationInfoHeader::new(other_ainfos))
            },
        ),
    )(input)
}

#[inline]
fn username_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    quoted_string(input)
}

fn username(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(tag("username"), equal, username_value),
        |(_, value)| AuthParameter::Username(value.into_owned()),
    )(input)
}

#[inline]
fn realm_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    quoted_string(input)
}

fn realm(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(tag("realm"), equal, realm_value),
        |(_, value)| AuthParameter::Realm(value.into_owned()),
    )(input)
}

fn nonce(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(tag("nonce"), equal, nonce_value),
        |(_, value)| AuthParameter::Nonce(value.into_owned()),
    )(input)
}

fn digest_uri_value(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    // TODO: Handle rquest-uri
    // delimited(ldquot, rquest_uri, rdquot)(input)
    quoted_string(input)
}

fn digest_uri(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(tag("uri"), equal, digest_uri_value),
        |(_, value)| AuthParameter::DigestUri(value.into_owned()),
    )(input)
}

fn request_digest(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        delimited(ldquot, recognize(many_m_n(32, 32, lhex)), rdquot),
        String::from_utf8_lossy,
    )(input)
}

fn dresponse(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(tag("response"), equal, request_digest),
        |(_, value)| AuthParameter::DResponse(value.into_owned()),
    )(input)
}

fn algorithm(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(
            tag("algorithm"),
            equal,
            alt((
                map(tag("MD5"), String::from_utf8_lossy),
                map(tag("MD5-sess"), String::from_utf8_lossy),
                token,
            )),
        ),
        |(_, value)| AuthParameter::Algorithm(Algorithm::new(value.into_owned())),
    )(input)
}

fn opaque(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(tag("opaque"), equal, quoted_string),
        |(_, value)| AuthParameter::Opaque(value.into_owned()),
    )(input)
}

#[inline]
fn auth_param_name(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn auth_param(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
    map(
        separated_pair(auth_param_name, equal, alt((token, quoted_string))),
        |(key, value)| AuthParameter::Other(key.to_string(), value.to_string()),
    )(input)
}

fn dig_resp(input: &[u8]) -> ParserResult<&[u8], AuthParameter> {
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

fn digest_response(input: &[u8]) -> ParserResult<&[u8], Vec<AuthParameter>> {
    map(
        pair(dig_resp, many0(preceded(comma, dig_resp))),
        |(first_param, mut other_params)| {
            other_params.insert(0, first_param);
            other_params
        },
    )(input)
}

#[inline]
fn auth_scheme(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    token(input)
}

fn auth_params(input: &[u8]) -> ParserResult<&[u8], Vec<AuthParameter>> {
    map(
        pair(auth_param, many0(preceded(comma, auth_param))),
        |(first_param, mut other_params)| {
            other_params.insert(0, first_param);
            other_params
        },
    )(input)
}

fn digest_credentials(input: &[u8]) -> ParserResult<&[u8], Credentials> {
    map(
        separated_pair(
            map(tag("Digest"), String::from_utf8_lossy),
            lws,
            digest_response,
        ),
        |(_, params)| Credentials::Digest(params),
    )(input)
}

fn other_response(input: &[u8]) -> ParserResult<&[u8], Credentials> {
    map(
        separated_pair(
            verify(auth_scheme, |s: &Cow<'_, str>| s != "Digest"),
            lws,
            auth_params,
        ),
        |(scheme, params)| Credentials::Other(scheme.to_string(), params),
    )(input)
}

fn credentials(input: &[u8]) -> ParserResult<&[u8], Credentials> {
    alt((digest_credentials, other_response))(input)
}

fn authorization(input: &[u8]) -> ParserResult<&[u8], Header> {
    context(
        "authorization",
        map(
            separated_pair(tag("Authorization"), hcolon, credentials),
            |(_, credentials)| Header::Authorization(AuthorizationHeader::new(credentials)),
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
            separated_pair(alt((tag("Call-ID"), tag("i"))), hcolon, callid),
            |(_, call_id)| Header::CallId(CallIdHeader::new(call_id.to_string())),
        ),
    )(input)
}

fn info_param(input: &[u8]) -> ParserResult<&[u8], CallInfoParameter> {
    map(
        alt((
            map(
                separated_pair(
                    map(tag("purpose"), String::from_utf8_lossy),
                    equal,
                    map(
                        alt((
                            map(tag("icon"), String::from_utf8_lossy),
                            map(tag("info"), String::from_utf8_lossy),
                            map(tag("card"), String::from_utf8_lossy),
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
            separated_pair(
                tag("Call-Info"),
                hcolon,
                pair(info, many0(preceded(comma, info))),
            ),
            |(_, (first_info, mut other_infos))| {
                other_infos.insert(0, first_info);
                Header::CallInfo(CallInfoHeader::new(other_infos))
            },
        ),
    )(input)
}

fn addr_spec(input: &[u8]) -> ParserResult<&[u8], Uri> {
    alt((sip_uri, sips_uri, map(absolute_uri, Uri::Absolute)))(input)
}

fn display_name(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    alt((
        quoted_string,
        map(recognize(many0(pair(token, lws))), String::from_utf8_lossy),
    ))(input)
}

fn name_addr(input: &[u8]) -> ParserResult<&[u8], NameAddress> {
    map(
        pair(opt(display_name), delimited(laquot, addr_spec, raquot)),
        |(display_name, uri)| NameAddress::new(uri, display_name.map(Into::into)),
    )(input)
}

fn c_p_q(input: &[u8]) -> ParserResult<&[u8], ContactParameter> {
    map(
        separated_pair(map(tag("q"), String::from_utf8_lossy), equal, qvalue),
        |(_, value)| ContactParameter::Q(value.to_string()),
    )(input)
}

#[inline]
fn delta_seconds(input: &[u8]) -> ParserResult<&[u8], u32> {
    map(
        recognize(many1(digit)),
        |digits| match String::from_utf8_lossy(digits).parse::<u32>() {
            Err(_) => u32::MAX, // If the value is larger than 2**32-1 (4294967295 seconds or 136 years), use the max
            Ok(value) => value,
        },
    )(input)
}

fn c_p_expires(input: &[u8]) -> ParserResult<&[u8], ContactParameter> {
    map(
        separated_pair(
            tag("expires"),
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
            separated_pair(
                alt((tag("Contact"), tag("m"))),
                hcolon,
                alt((
                    map(star, |_| ContactHeader::Any),
                    map(
                        pair(contact_param, many0(preceded(comma, contact_param))),
                        |(first_contact, mut other_contacts)| {
                            other_contacts.insert(0, first_contact);
                            ContactHeader::Contacts(other_contacts)
                        },
                    ),
                )),
            ),
            |(_, contact_header)| Header::Contact(contact_header),
        ),
    )(input)
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
            call_info,
            contact,
        )),
    )(input)
}
