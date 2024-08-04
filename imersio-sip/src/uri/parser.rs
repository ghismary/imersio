use std::{borrow::Cow, num::NonZeroU16};

use crate::{
    parser::{
        alpha, digit, escaped, hex_digit, is_reserved, is_unreserved, reserved, take1, token, ttl,
        unreserved, ParserResult,
    },
    utils::{extend_vec, has_unique_elements},
    AbsoluteUri, HostPort, Uri, UriHeaders, UriParameters, UserInfo,
};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::digit1,
    combinator::{cut, map, map_opt, opt, recognize, verify},
    error::context,
    multi::{many0, many1, many_m_n},
    sequence::{pair, preceded, separated_pair, tuple},
    ParseTo,
};

use super::{sip_uri::SipUri, uri_scheme::UriScheme};

#[inline]
pub(crate) fn is_user_unreserved(b: u8) -> bool {
    b"&=+$,;?/".contains(&b)
}

fn user_unreserved(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_user_unreserved(*b)))(input)
}

fn user(input: &[u8]) -> ParserResult<&[u8], String> {
    context(
        "user",
        map(many1(alt((unreserved, escaped, user_unreserved))), |user| {
            user.iter()
                .map(|b| String::from_utf8_lossy(b))
                .collect::<String>()
        }),
    )(input)
}

#[inline]
pub(crate) fn is_password_special_char(b: u8) -> bool {
    b"&=+$,".contains(&b)
}

fn password_special_char(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_password_special_char(*b)))(input)
}

fn password(input: &[u8]) -> ParserResult<&[u8], String> {
    context(
        "password",
        map(
            many0(alt((unreserved, escaped, password_special_char))),
            |password| {
                password
                    .iter()
                    .map(|b| String::from_utf8_lossy(b))
                    .collect::<String>()
            },
        ),
    )(input)
}

fn userinfo(input: &[u8]) -> ParserResult<&[u8], UserInfo> {
    map(
        tuple((
            user, // TODO: alt((user, telephone_subscriber)),
            opt(preceded(tag(":"), password)),
            tag("@"),
        )),
        |(user, password, _)| UserInfo::new(user, password),
    )(input)
}

fn is_valid_hostname(input: &str) -> bool {
    let mut labels: Vec<&str> = input.split('.').collect();
    // A valid hostname may end by '.', if this is the case the last label
    // will be empty, and so we remove before further processing.
    if labels.last().is_some_and(|label| label.is_empty()) {
        labels.pop();
    }
    // If nothing remains, this is not valid.
    if labels.is_empty() {
        return false;
    }
    // All other labels must not be empty.
    if labels.iter().any(|label| label.is_empty()) {
        return false;
    }
    // The '-' must not be located at the begining or at the end of a
    // label.
    if labels
        .iter()
        .all(|label| label.starts_with('-') || label.ends_with('-'))
    {
        return false;
    }
    labels
        .pop()
        .is_some_and(|label| label.as_bytes()[0].is_ascii_alphabetic())
}

fn hostname(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    context(
        "hostname",
        verify(
            map(
                recognize(many1(verify(take1, |b| {
                    b.is_ascii_alphanumeric() || b"-.".contains(b)
                }))),
                String::from_utf8_lossy,
            ),
            is_valid_hostname,
        ),
    )(input)
}

#[inline]
fn is_valid_ipv4_address_number(input: &str) -> bool {
    input.parse::<u8>().is_ok()
}

fn ipv4_address_number(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(recognize(many_m_n(1, 3, digit)), String::from_utf8_lossy)(input)
}

fn ipv4_address(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    context(
        "ipv4_address",
        map(
            recognize(tuple((
                verify(ipv4_address_number, is_valid_ipv4_address_number),
                tag("."),
                verify(ipv4_address_number, is_valid_ipv4_address_number),
                tag("."),
                verify(ipv4_address_number, is_valid_ipv4_address_number),
                tag("."),
                verify(ipv4_address_number, is_valid_ipv4_address_number),
            ))),
            String::from_utf8_lossy,
        ),
    )(input)
}

fn hex4(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(many_m_n(1, 4, hex_digit))(input)
}

fn hexseq(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(hex4, many0(pair(tag(":"), hex4))))(input)
}

fn hexpart(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(alt((
        hexseq,
        recognize(tuple((hexseq, tag("::"), hexseq))),
        recognize(pair(tag("::"), hexseq)),
    )))(input)
}

fn ipv6_address(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    context(
        "ipv6_address",
        recognize(pair(hexpart, opt(pair(tag(":"), ipv4_address)))),
    )(input)
}

fn ipv6_reference(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    context(
        "ipv6_reference",
        map(
            recognize(tuple((tag("["), ipv6_address, tag("]")))),
            String::from_utf8_lossy,
        ),
    )(input)
}

pub(crate) fn host(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    context("host", alt((hostname, ipv4_address, ipv6_reference)))(input)
}

fn port(input: &[u8]) -> ParserResult<&[u8], NonZeroU16> {
    let mut port_u16 = map_opt(digit1, |s: &[u8]| s.parse_to());
    port_u16(input)
}

fn hostport(input: &[u8]) -> ParserResult<&[u8], HostPort> {
    map(pair(host, opt(preceded(tag(":"), port))), |(host, port)| {
        HostPort::new(host, port)
    })(input)
}

fn transport_param(input: &[u8]) -> ParserResult<&[u8], (Cow<'_, str>, Cow<'_, str>)> {
    context(
        "transport_param",
        map(
            separated_pair(tag("transport"), tag("="), token),
            |(name, value)| (String::from_utf8_lossy(name), value),
        ),
    )(input)
}

fn user_param(input: &[u8]) -> ParserResult<&[u8], (Cow<'_, str>, Cow<'_, str>)> {
    context(
        "user_param",
        map(
            separated_pair(tag("user"), tag("="), token),
            |(name, value)| (String::from_utf8_lossy(name), value),
        ),
    )(input)
}

fn method_param(input: &[u8]) -> ParserResult<&[u8], (Cow<'_, str>, Cow<'_, str>)> {
    context(
        "method_param",
        map(
            separated_pair(tag("method"), tag("="), token),
            |(name, value)| (String::from_utf8_lossy(name), value),
        ),
    )(input)
}

fn ttl_param(input: &[u8]) -> ParserResult<&[u8], (Cow<'_, str>, Cow<'_, str>)> {
    context(
        "ttl_param",
        map(
            separated_pair(tag("ttl"), tag("="), ttl),
            |(name, value)| {
                (
                    String::from_utf8_lossy(name),
                    String::from_utf8_lossy(value),
                )
            },
        ),
    )(input)
}

fn maddr_param(input: &[u8]) -> ParserResult<&[u8], (Cow<'_, str>, Cow<'_, str>)> {
    context(
        "maddr_param",
        map(
            separated_pair(tag("maddr"), tag("="), host),
            |(name, value)| (String::from_utf8_lossy(name), value),
        ),
    )(input)
}

#[inline]
fn lr_param(input: &[u8]) -> ParserResult<&[u8], (Cow<'_, str>, Cow<'_, str>)> {
    map(context("lr_param", tag("lr")), |name| {
        (String::from_utf8_lossy(name), Cow::from(""))
    })(input)
}

#[inline]
pub(crate) fn is_param_unreserved(b: u8) -> bool {
    b"[]/:&+$".contains(&b)
}

fn param_unreserved(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_param_unreserved(*b)))(input)
}

fn paramchar(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    alt((param_unreserved, unreserved, escaped))(input)
}

fn pname(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    context(
        "pname",
        map(many1(paramchar), |pname| {
            pname
                .iter()
                .map(|b| String::from_utf8_lossy(b))
                .collect::<String>()
                .into()
        }),
    )(input)
}

fn pvalue(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    context(
        "pvalue",
        map(many1(paramchar), |pvalue| {
            pvalue
                .iter()
                .map(|b| String::from_utf8_lossy(b))
                .collect::<String>()
                .into()
        }),
    )(input)
}

fn other_param(input: &[u8]) -> ParserResult<&[u8], (Cow<'_, str>, Cow<'_, str>)> {
    context(
        "other_param",
        map(
            pair(pname, opt(preceded(tag("="), pvalue))),
            |(name, value)| (name, value.unwrap_or_default()),
        ),
    )(input)
}

fn uri_parameter(input: &[u8]) -> ParserResult<&[u8], (Cow<'_, str>, Cow<'_, str>)> {
    alt((
        transport_param,
        user_param,
        method_param,
        ttl_param,
        maddr_param,
        lr_param,
        other_param,
    ))(input)
}

fn uri_parameters(input: &[u8]) -> ParserResult<&[u8], UriParameters> {
    context(
        "uri_parameters",
        map(many0(preceded(tag(";"), uri_parameter)), |parameters| {
            UriParameters::new(
                parameters
                    .into_iter()
                    .map(|(k, v)| (k, if v.is_empty() { None } else { Some(v) }))
                    .collect(),
            )
        }),
    )(input)
}

#[inline]
pub(crate) fn is_hnv_unreserved(b: u8) -> bool {
    b"[]/?:+$".contains(&b)
}

fn hnv_unreserved(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_hnv_unreserved(*b)))(input)
}

fn hname(input: &[u8]) -> ParserResult<&[u8], String> {
    context(
        "hname",
        map(many1(alt((hnv_unreserved, unreserved, escaped))), |name| {
            name.iter()
                .map(|b| String::from_utf8_lossy(b))
                .collect::<String>()
        }),
    )(input)
}

fn hvalue(input: &[u8]) -> ParserResult<&[u8], String> {
    context(
        "hvalue",
        map(many0(alt((hnv_unreserved, unreserved, escaped))), |value| {
            value
                .iter()
                .map(|b| String::from_utf8_lossy(b))
                .collect::<String>()
        }),
    )(input)
}

fn header(input: &[u8]) -> ParserResult<&[u8], (String, String)> {
    separated_pair(hname, tag("="), hvalue)(input)
}

fn headers(input: &[u8]) -> ParserResult<&[u8], UriHeaders> {
    context(
        "headers",
        map(
            pair(
                preceded(tag("?"), header),
                many0(preceded(tag("&"), header)),
            ),
            |(first_header, other_headers)| {
                UriHeaders::new(extend_vec(first_header, other_headers))
            },
        ),
    )(input)
}

pub(crate) fn sip_uri(input: &[u8]) -> ParserResult<&[u8], Uri> {
    context(
        "sip",
        preceded(
            tag_no_case("sip:"),
            cut(map(
                tuple((
                    opt(userinfo),
                    hostport,
                    cut(verify(uri_parameters, |params| {
                        has_unique_elements(params.iter().map(|p| &p.0))
                    })),
                    opt(headers),
                )),
                |(userinfo, hostport, parameters, headers)| {
                    Uri::Sip(SipUri::new(
                        UriScheme::SIP,
                        userinfo,
                        hostport,
                        parameters,
                        headers.unwrap_or_default(),
                    ))
                },
            )),
        ),
    )(input)
}

pub(crate) fn sips_uri(input: &[u8]) -> ParserResult<&[u8], Uri> {
    context(
        "sips_uri",
        preceded(
            tag_no_case("sips:"),
            cut(map(
                tuple((
                    opt(userinfo),
                    hostport,
                    cut(verify(uri_parameters, |params| {
                        has_unique_elements(params.iter().map(|p| &p.0))
                    })),
                    opt(headers),
                )),
                |(userinfo, hostport, parameters, headers)| {
                    Uri::Sip(SipUri::new(
                        UriScheme::SIPS,
                        userinfo,
                        hostport,
                        parameters,
                        headers.unwrap_or_default(),
                    ))
                },
            )),
        ),
    )(input)
}

fn uric(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    alt((reserved, unreserved, escaped))(input)
}

fn uric_no_slash(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| {
        is_reserved(*b) || is_unreserved(*b) || b";?:@&=+$,".contains(b)
    }))(input)
}

fn scheme_special_char(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| b"+-.".contains(b)))(input)
}

fn scheme(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    context(
        "scheme",
        map(
            recognize(pair(alpha, many0(alt((alpha, digit, scheme_special_char))))),
            String::from_utf8_lossy,
        ),
    )(input)
}

fn opaque_part(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    map(
        recognize(pair(uric_no_slash, many0(uric))),
        String::from_utf8_lossy,
    )(input)
}

pub(crate) fn absolute_uri(input: &[u8]) -> ParserResult<&[u8], AbsoluteUri> {
    context(
        "absolute_uri",
        map(
            separated_pair(scheme, tag(":"), opaque_part),
            |(scheme, opaque_part)| {
                AbsoluteUri::new(
                    UriScheme::Other(scheme.into_owned()),
                    opaque_part,
                    UriParameters::default(),
                    UriHeaders::default(),
                )
            },
        ),
    )(input)
}

pub(crate) fn request_uri(input: &[u8]) -> ParserResult<&[u8], Uri> {
    context(
        "uri",
        alt((sip_uri, sips_uri, map(absolute_uri, Uri::Absolute))),
    )(input)
}
