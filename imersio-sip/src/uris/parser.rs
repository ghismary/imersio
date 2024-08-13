use crate::{
    parser::{
        alpha, digit, escaped, hex_digit, is_reserved, is_unreserved, reserved, take1, token, ttl,
        unreserved, ParserResult,
    },
    utils::has_unique_elements,
    AbsoluteUri, Host, Uri, UriHeaders, UriParameters, UserInfo,
};
use std::net::IpAddr;

use nom::sequence::delimited;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::{cut, map, opt, recognize, verify},
    error::context,
    multi::{many0, many1, many_m_n, separated_list1},
    sequence::{pair, preceded, separated_pair, tuple},
    ParseTo,
};

use super::{sip_uri::SipUri, uri_scheme::UriScheme};

#[inline]
pub(crate) fn is_user_unreserved(c: char) -> bool {
    "&=+$,;?/".contains(c)
}

fn user_unreserved(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| is_user_unreserved(*c))(input)
}

fn user(input: &str) -> ParserResult<&str, String> {
    context(
        "user",
        map(many1(alt((unreserved, escaped, user_unreserved))), |user| {
            user.iter().collect::<String>()
        }),
    )(input)
}

#[inline]
pub(crate) fn is_password_special_char(c: char) -> bool {
    "&=+$,".contains(c)
}

fn password_special_char(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| is_password_special_char(*c))(input)
}

fn password(input: &str) -> ParserResult<&str, String> {
    context(
        "password",
        map(
            many0(alt((unreserved, escaped, password_special_char))),
            |password| password.iter().collect::<String>(),
        ),
    )(input)
}

fn userinfo(input: &str) -> ParserResult<&str, UserInfo> {
    context(
        "userinfo",
        map(
            tuple((
                user, // TODO: alt((user, telephone_subscriber)),
                opt(preceded(tag(":"), password)),
                tag("@"),
            )),
            |(user, password, _)| UserInfo::new(user, password),
        ),
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
    // The '-' must not be located at the beginning or at the end of a
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

fn hostname(input: &str) -> ParserResult<&str, Host> {
    context(
        "hostname",
        map(
            verify(
                recognize(many1(verify(take1, |c| {
                    c.is_ascii_alphanumeric() || "-.".contains(*c)
                }))),
                is_valid_hostname,
            ),
            |name| Host::Name(name.into()),
        ),
    )(input)
}

#[inline]
fn is_valid_ipv4_address_number(input: &str) -> bool {
    input.parse::<u8>().is_ok()
}

fn ipv4_address_number(input: &str) -> ParserResult<&str, &str> {
    recognize(many_m_n(1, 3, digit))(input)
}

pub(crate) fn ipv4_address(input: &str) -> ParserResult<&str, IpAddr> {
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
            |ipv4| ipv4.parse().unwrap(),
        ),
    )(input)
}

fn hex4(input: &str) -> ParserResult<&str, &str> {
    recognize(many_m_n(1, 4, hex_digit))(input)
}

fn hexseq(input: &str) -> ParserResult<&str, &str> {
    recognize(pair(hex4, many0(pair(tag(":"), hex4))))(input)
}

fn hexpart(input: &str) -> ParserResult<&str, &str> {
    recognize(alt((
        hexseq,
        recognize(tuple((hexseq, tag("::"), hexseq))),
        recognize(pair(tag("::"), hexseq)),
    )))(input)
}

pub(crate) fn ipv6_address(input: &str) -> ParserResult<&str, IpAddr> {
    context(
        "ipv6_address",
        map(
            recognize(pair(hexpart, opt(pair(tag(":"), ipv4_address)))),
            |ipv6| ipv6.parse().unwrap(),
        ),
    )(input)
}

fn ipv6_reference(input: &str) -> ParserResult<&str, IpAddr> {
    context(
        "ipv6_reference",
        delimited(tag("["), ipv6_address, tag("]")),
    )(input)
}

pub(crate) fn host(input: &str) -> ParserResult<&str, Host> {
    context(
        "host",
        alt((
            hostname,
            map(ipv4_address, Host::Ip),
            map(ipv6_reference, Host::Ip),
        )),
    )(input)
}

pub(crate) fn port(input: &str) -> ParserResult<&str, u16> {
    context(
        "port",
        map(recognize(many1(digit)), |digits| digits.parse_to().unwrap()),
    )(input)
}

pub(crate) fn hostport(input: &str) -> ParserResult<&str, (Host, Option<u16>)> {
    context("hostport", pair(host, opt(preceded(tag(":"), port))))(input)
}

fn transport_param(input: &str) -> ParserResult<&str, (String, String)> {
    context(
        "transport_param",
        map(
            separated_pair(tag("transport"), tag("="), token),
            |(name, value)| (name.to_string(), value.to_string()),
        ),
    )(input)
}

fn user_param(input: &str) -> ParserResult<&str, (String, String)> {
    context(
        "user_param",
        map(
            separated_pair(tag("user"), tag("="), token),
            |(name, value)| (name.to_string(), value.to_string()),
        ),
    )(input)
}

fn method_param(input: &str) -> ParserResult<&str, (String, String)> {
    context(
        "method_param",
        map(
            separated_pair(tag("method"), tag("="), token),
            |(name, value)| (name.to_string(), value.to_string()),
        ),
    )(input)
}

fn ttl_param(input: &str) -> ParserResult<&str, (String, String)> {
    context(
        "ttl_param",
        map(
            separated_pair(tag("ttl"), tag("="), ttl),
            |(name, value)| (name.to_string(), value.to_string()),
        ),
    )(input)
}

fn maddr_param(input: &str) -> ParserResult<&str, (String, String)> {
    context(
        "maddr_param",
        map(
            separated_pair(tag("maddr"), tag("="), host),
            |(name, value)| (name.to_string(), value.to_string()),
        ),
    )(input)
}

#[inline]
fn lr_param(input: &str) -> ParserResult<&str, (String, String)> {
    context(
        "lr_param",
        map(tag("lr"), |name: &str| (name.to_string(), "".to_string())),
    )(input)
}

#[inline]
pub(crate) fn is_param_unreserved(c: char) -> bool {
    "[]/:&+$".contains(c)
}

fn param_unreserved(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| is_param_unreserved(*c))(input)
}

fn paramchar(input: &str) -> ParserResult<&str, char> {
    alt((param_unreserved, unreserved, escaped))(input)
}

fn pname(input: &str) -> ParserResult<&str, String> {
    context(
        "pname",
        map(many1(paramchar), |pname| pname.iter().collect::<String>()),
    )(input)
}

fn pvalue(input: &str) -> ParserResult<&str, String> {
    context(
        "pvalue",
        map(many1(paramchar), |pvalue| pvalue.iter().collect::<String>()),
    )(input)
}

fn other_param(input: &str) -> ParserResult<&str, (String, String)> {
    context(
        "other_param",
        map(
            pair(pname, opt(preceded(tag("="), pvalue))),
            |(name, value)| (name, value.unwrap_or_default()),
        ),
    )(input)
}

fn uri_parameter(input: &str) -> ParserResult<&str, (String, String)> {
    context(
        "uri_parameter",
        alt((
            transport_param,
            user_param,
            method_param,
            ttl_param,
            maddr_param,
            lr_param,
            other_param,
        )),
    )(input)
}

fn uri_parameters(input: &str) -> ParserResult<&str, UriParameters> {
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
pub(crate) fn is_hnv_unreserved(c: char) -> bool {
    "[]/?:+$".contains(c)
}

fn hnv_unreserved(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| is_hnv_unreserved(*c))(input)
}

fn hname(input: &str) -> ParserResult<&str, String> {
    context(
        "hname",
        map(many1(alt((hnv_unreserved, unreserved, escaped))), |name| {
            name.iter().collect::<String>()
        }),
    )(input)
}

fn hvalue(input: &str) -> ParserResult<&str, String> {
    context(
        "hvalue",
        map(many0(alt((hnv_unreserved, unreserved, escaped))), |value| {
            value.iter().collect::<String>()
        }),
    )(input)
}

fn header(input: &str) -> ParserResult<&str, (String, String)> {
    context("header", separated_pair(hname, tag("="), hvalue))(input)
}

fn headers(input: &str) -> ParserResult<&str, UriHeaders> {
    context(
        "headers",
        map(
            preceded(tag("?"), separated_list1(tag("&"), header)),
            UriHeaders::new,
        ),
    )(input)
}

pub(crate) fn sip_uri(input: &str) -> ParserResult<&str, Uri> {
    context(
        "sip_uri",
        map(
            pair(
                alt((
                    map(tag_no_case("sip:"), |_| UriScheme::SIP),
                    map(tag_no_case("sips:"), |_| UriScheme::SIPS),
                )),
                cut(tuple((
                    opt(userinfo),
                    hostport,
                    cut(verify(uri_parameters, |params| {
                        has_unique_elements(params.iter().map(|p| &p.0))
                    })),
                    opt(headers),
                ))),
            ),
            |(scheme, (userinfo, (host, port), parameters, headers))| {
                Uri::Sip(SipUri::new(
                    scheme,
                    userinfo,
                    host,
                    port,
                    parameters,
                    headers.unwrap_or_default(),
                ))
            },
        ),
    )(input)
}

fn uric(input: &str) -> ParserResult<&str, char> {
    alt((reserved, unreserved, escaped))(input)
}

fn uric_no_slash(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| {
        is_reserved(*c) || is_unreserved(*c) || ";?:@&=+$,".contains(*c)
    })(input)
}

fn scheme_special_char(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| "+-.".contains(*c))(input)
}

fn scheme(input: &str) -> ParserResult<&str, &str> {
    context(
        "scheme",
        recognize(pair(alpha, many0(alt((alpha, digit, scheme_special_char))))),
    )(input)
}

fn opaque_part(input: &str) -> ParserResult<&str, &str> {
    recognize(pair(uric_no_slash, many0(uric)))(input)
}

pub(crate) fn absolute_uri(input: &str) -> ParserResult<&str, AbsoluteUri> {
    context(
        "absolute_uri",
        map(
            separated_pair(scheme, tag(":"), opaque_part),
            |(scheme, opaque_part)| {
                AbsoluteUri::new(
                    UriScheme::Other(scheme.to_string()),
                    opaque_part,
                    UriParameters::default(),
                    UriHeaders::default(),
                )
            },
        ),
    )(input)
}

pub(crate) fn request_uri(input: &str) -> ParserResult<&str, Uri> {
    context("uri", alt((sip_uri, map(absolute_uri, Uri::Absolute))))(input)
}
