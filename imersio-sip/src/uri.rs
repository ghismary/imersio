//! TODO

use std::{hash::Hash, num::NonZeroU16, ops::Deref, str::FromStr};

use crate::{parser::is_unreserved, utils::escape, Error};

/// Representation of an URI, whether a SIP URI or an absolute URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Uri {
    /// A sip: or sips: URI
    Sip(SipUri),
    /// Any other URI
    Absolute(AbsoluteUri),
}

/// Representation of a SIP URI.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct SipUri {
    scheme: Scheme,
    userinfo: Option<UserInfo>,
    hostport: HostPort,
    parameters: Parameters,
    headers: Headers,
}

impl SipUri {
    /// Get the `Scheme` of the `SipUri`.
    pub fn get_scheme(&self) -> &Scheme {
        &self.scheme
    }

    /// Get the `UserInfo` of the `SipUri`.
    pub fn get_userinfo(&self) -> Option<&UserInfo> {
        self.userinfo.as_ref()
    }

    /// Get the `HostPort` of the `SipUri`.
    pub fn get_hostport(&self) -> &HostPort {
        &self.hostport
    }

    /// Get the `Parameters` of the `SipUri`.
    pub fn get_parameters(&self) -> &Parameters {
        &self.parameters
    }

    /// Get the `Headers` of the `SipUri`.
    pub fn get_headers(&self) -> &Headers {
        &self.headers
    }
}

impl std::fmt::Display for SipUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}{}{}{}{}{}{}",
            self.scheme,
            if let Some(userinfo) = &self.userinfo {
                format!("{}", userinfo)
            } else {
                "".to_owned()
            },
            if self.userinfo.is_some() { "@" } else { "" },
            self.hostport,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters,
            if self.headers.is_empty() { "" } else { "?" },
            self.headers
        )
    }
}

impl PartialEq<&SipUri> for SipUri {
    fn eq(&self, other: &&SipUri) -> bool {
        self == *other
    }
}

impl PartialEq<SipUri> for &SipUri {
    fn eq(&self, other: &SipUri) -> bool {
        *self == other
    }
}

/// Representation of an absolute URI.
///
/// As of now, only the scheme is distinguished for the rest of the URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AbsoluteUri {
    scheme: Scheme,
    opaque_part: String,
    parameters: Parameters,
    headers: Headers,
}

impl AbsoluteUri {
    /// Get the `Scheme` of the `AbsoluteUri`.
    pub fn get_scheme(&self) -> &Scheme {
        &self.scheme
    }

    /// Get the `Parameters` of the `AbsoluteUri`.
    pub fn get_parameters(&self) -> &Parameters {
        &self.parameters
    }

    /// Get the `Headers` of the `AbsoluteUri`.
    pub fn get_headers(&self) -> &Headers {
        &self.headers
    }
}

impl std::fmt::Display for AbsoluteUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.scheme, self.opaque_part)
    }
}

impl PartialEq<&AbsoluteUri> for AbsoluteUri {
    fn eq(&self, other: &&AbsoluteUri) -> bool {
        self == *other
    }
}

impl PartialEq<AbsoluteUri> for &AbsoluteUri {
    fn eq(&self, other: &AbsoluteUri) -> bool {
        *self == other
    }
}

impl Uri {
    /// Try to create a `Uri` from a slice of bytes.
    #[inline]
    pub fn from_bytes(input: &[u8]) -> Result<Uri, Error> {
        parse_uri(input)
    }

    /// Gets the `Uri` as an `AbsoluteUri`.
    ///
    /// It returns None if the uri is not an `AbsoluteUri`.
    pub fn as_absolute_uri(&self) -> Option<&AbsoluteUri> {
        match self {
            Uri::Absolute(uri) => Some(uri),
            _ => None,
        }
    }

    /// Gets the `Uri` as a `SipUri`.
    ///
    /// It returns None if the uri is not a `SipUri`.
    pub fn as_sip_uri(&self) -> Option<&SipUri> {
        match self {
            Uri::Sip(uri) => Some(uri),
            _ => None,
        }
    }

    /// Tells whether this `Uri` is a SIP URI.
    pub fn is_sip(&self) -> bool {
        matches!(self, Uri::Sip(_))
    }

    /// Telss whether this `Uri` is secure or not.
    pub fn is_secure(&self) -> bool {
        match self {
            Uri::Sip(uri) => uri.get_scheme() == Scheme::SIPS,
            _ => false,
        }
    }

    /// Get the `Scheme` of the URI.
    pub fn get_scheme(&self) -> &Scheme {
        match self {
            Uri::Sip(uri) => uri.get_scheme(),
            Uri::Absolute(uri) => uri.get_scheme(),
        }
    }

    /// Get the user from the URI.
    pub fn get_user(&self) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.get_userinfo().map(|ui| ui.get_user()),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the password from the URI.
    pub fn get_password(&self) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.get_userinfo().and_then(|ui| ui.get_password()),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the host from the URI.
    pub fn get_host(&self) -> &str {
        match self {
            Uri::Sip(uri) => uri.get_hostport().get_host(),
            Uri::Absolute(uri) => &uri.opaque_part,
        }
    }

    /// Get the port from the URI.
    pub fn get_port(&self) -> Option<NonZeroU16> {
        match self {
            Uri::Sip(uri) => uri.get_hostport().get_port(),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the `Parameters` of the URI.
    pub fn get_parameters(&self) -> &Parameters {
        match self {
            Uri::Sip(uri) => uri.get_parameters(),
            Uri::Absolute(uri) => uri.get_parameters(),
        }
    }

    /// Get a parameter value of the URI given its name.
    pub fn get_parameter(&self, name: &str) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.get_parameters().get(name),
            Uri::Absolute(_) => None,
        }
    }

    /// Get the `Headers` of the URI.
    pub fn get_headers(&self) -> &Headers {
        match self {
            Uri::Sip(uri) => uri.get_headers(),
            Uri::Absolute(uri) => uri.get_headers(),
        }
    }

    /// Get a header value of the URI given its name.
    pub fn get_header(&self, name: &str) -> Option<&str> {
        match self {
            Uri::Sip(uri) => uri.get_headers().get(name),
            Uri::Absolute(_) => None,
        }
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Uri::Sip(uri) => uri.to_string(),
                Uri::Absolute(uri) => uri.to_string(),
            }
        )
    }
}

impl FromStr for Uri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uri::from_bytes(s.as_bytes())
    }
}

impl Default for Uri {
    fn default() -> Self {
        Uri::Sip(SipUri::default())
    }
}

impl PartialEq<&Uri> for Uri {
    fn eq(&self, other: &&Uri) -> bool {
        self == *other
    }
}

impl PartialEq<Uri> for &Uri {
    fn eq(&self, other: &Uri) -> bool {
        *self == other
    }
}

/// Representation of the scheme of an URI.
#[derive(Clone, Debug)]
pub enum Scheme {
    /// SIP protocol scheme.
    Sip,
    /// SIPS protocol scheme.
    Sips,
    /// Any other protocol scheme.
    Other(String),
}

impl Scheme {
    /// SIP protocol scheme.
    pub const SIP: Scheme = Scheme::Sip;

    /// SIPS protocol scheme.
    pub const SIPS: Scheme = Scheme::Sips;

    /// Get a str representation of the scheme.
    pub fn as_str(&self) -> &str {
        match self {
            Scheme::Sip => "sip",
            Scheme::Sips => "sips",
            Scheme::Other(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for Scheme {
    fn default() -> Self {
        Scheme::SIP
    }
}

impl AsRef<str> for Scheme {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for Scheme {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (&Scheme::Sip, &Scheme::Sip) => true,
            (&Scheme::Sips, &Scheme::Sips) => true,
            (Scheme::Other(a), Scheme::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialEq<str> for Scheme {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<Scheme> for str {
    fn eq(&self, other: &Scheme) -> bool {
        other == self
    }
}

impl PartialEq<&Scheme> for Scheme {
    fn eq(&self, other: &&Scheme) -> bool {
        self == *other
    }
}

impl PartialEq<Scheme> for &Scheme {
    fn eq(&self, other: &Scheme) -> bool {
        *self == other
    }
}

impl Eq for Scheme {}

impl Hash for Scheme {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Scheme::Sip => {
                state.write_u8(1);
            }
            Scheme::Sips => {
                state.write_u8(2);
            }
            Scheme::Other(value) => {
                state.write_u8(3);
                value.to_ascii_lowercase().hash(state);
            }
        }
    }
}

fn parse_uri(input: &[u8]) -> Result<Uri, Error> {
    match parser::uri(input) {
        Ok((rest, uri)) => {
            if !rest.is_empty() {
                Err(Error::RemainingUnparsedData)
            } else {
                Ok(uri)
            }
        }
        Err(e) => Err(Error::InvalidUri(e.to_string())),
    }
}

/// Representation of an userinfo of a SIP URI.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UserInfo {
    user: String,
    password: Option<String>,
}

impl UserInfo {
    /// Get the user part of the `UserInfo`.
    pub fn get_user(&self) -> &str {
        &self.user
    }

    /// Get the password part of the `UserInfo`.
    pub fn get_password(&self) -> Option<&str> {
        self.password.as_deref()
    }
}

impl std::fmt::Display for UserInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            escape(&self.user, |b| {
                is_unreserved(b) || parser::is_user_unreserved(b)
            }),
            if self.password.is_some() { ":" } else { "" },
            escape(self.password.as_deref().unwrap_or_default(), |b| {
                is_unreserved(b) || parser::is_password_special_char(b)
            })
        )
    }
}

impl PartialEq<&UserInfo> for UserInfo {
    fn eq(&self, other: &&UserInfo) -> bool {
        self == *other
    }
}

impl PartialEq<UserInfo> for &UserInfo {
    fn eq(&self, other: &UserInfo) -> bool {
        *self == other
    }
}

/// Representation of a hostport of a SIP URI.
#[derive(Clone, Debug)]
pub struct HostPort {
    host: String,
    port: Option<NonZeroU16>,
}

impl HostPort {
    /// Get the host part of the `HostPort`.
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Get the port part of the `HostPort`.
    pub fn get_port(&self) -> Option<NonZeroU16> {
        self.port
    }
}

impl std::fmt::Display for HostPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.host,
            if self.port.is_some() { ":" } else { "" },
            self.port.map(|p| format!("{p}")).unwrap_or_default()
        )
    }
}

impl Default for HostPort {
    fn default() -> Self {
        HostPort {
            host: "localhost".to_string(),
            port: None,
        }
    }
}

impl PartialEq for HostPort {
    fn eq(&self, other: &Self) -> bool {
        self.host.eq_ignore_ascii_case(&other.host) && self.port == other.port
    }
}

impl PartialEq<&HostPort> for HostPort {
    fn eq(&self, other: &&HostPort) -> bool {
        self == *other
    }
}

impl PartialEq<HostPort> for &HostPort {
    fn eq(&self, other: &HostPort) -> bool {
        *self == other
    }
}

impl Eq for HostPort {}

impl Hash for HostPort {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.host.to_ascii_lowercase().hash(state);
        self.port.hash(state);
    }
}

/// Representation of an URI parameter list.
#[derive(Clone, Debug, Default)]
pub struct Parameters(Vec<(String, Option<String>)>);

impl Parameters {
    /// Tells whether the parameters list is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of parameters.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Tells whether the parameters list contains a parameter with the given
    /// name.
    pub fn contains(&self, name: &str) -> bool {
        self.iter().any(|(n, _)| n == name)
    }

    /// Gets the parameter corresponding to the given name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .and_then(|(_, v)| v.as_deref())
    }
}

impl std::fmt::Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}{}{}",
                        escape(k, |b| {
                            is_unreserved(b) || parser::is_param_unreserved(b)
                        }),
                        if v.is_some() { "=" } else { "" },
                        escape(v.as_deref().unwrap_or_default(), |b| {
                            is_unreserved(b) || parser::is_param_unreserved(b)
                        })
                    )
                })
                .collect::<Vec<String>>()
                .join(";"),
        )
    }
}

impl Deref for Parameters {
    type Target = Vec<(String, Option<String>)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Parameters {
    fn eq(&self, other: &Self) -> bool {
        for (sk, sv) in &self.0 {
            for (ok, ov) in &other.0 {
                if sk.eq_ignore_ascii_case(ok)
                    && sv.as_ref().map(|s| s.to_ascii_lowercase())
                        != ov.as_ref().map(|s| s.to_ascii_lowercase())
                {
                    return false;
                }
            }
        }

        let stransport = self.get("transport");
        let otransport = other.get("transport");
        match (stransport, otransport) {
            (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}

impl PartialEq<&Parameters> for Parameters {
    fn eq(&self, other: &&Parameters) -> bool {
        self == *other
    }
}

impl PartialEq<Parameters> for &Parameters {
    fn eq(&self, other: &Parameters) -> bool {
        *self == other
    }
}

impl Eq for Parameters {}

impl Hash for Parameters {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut sorted_params: Vec<(String, Option<String>)> = self
            .0
            .iter()
            .map(|(key, value)| {
                (
                    key.to_ascii_lowercase(),
                    value.as_ref().map(|value| value.to_ascii_lowercase()),
                )
            })
            .collect();
        sorted_params.sort_by(|(a, _), (b, _)| a.cmp(b));
        sorted_params.hash(state)
    }
}

/// Representation of an URI header list.
#[derive(Clone, Debug, Default)]
pub struct Headers(Vec<(String, String)>);

impl Headers {
    /// Tells whether the headers list is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of headers.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Tells whether the headers list contains a header with the given name.
    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|(n, _)| n == name)
    }

    /// Gets the header corresponding to the given name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_ref())
    }
}

impl std::fmt::Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        escape(k, |b| { is_unreserved(b) || parser::is_hnv_unreserved(b) }),
                        escape(v, |b| { is_unreserved(b) || parser::is_hnv_unreserved(b) })
                    )
                })
                .collect::<Vec<String>>()
                .join("&"),
        )
    }
}

impl PartialEq for Headers {
    fn eq(&self, other: &Self) -> bool {
        for (sk, sv) in &self.0 {
            if let Some(ov) = other.get(sk) {
                if sv != ov {
                    return false;
                }
            } else {
                return false;
            }
        }

        for (ok, ov) in &other.0 {
            if let Some(sv) = self.get(ok) {
                if ov != sv {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl PartialEq<&Headers> for Headers {
    fn eq(&self, other: &&Headers) -> bool {
        self == *other
    }
}

impl PartialEq<Headers> for &Headers {
    fn eq(&self, other: &Headers) -> bool {
        *self == other
    }
}

impl Eq for Headers {}

impl Hash for Headers {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut sorted_headers: Vec<(String, String)> = self
            .0
            .iter()
            .map(|(key, value)| (key.to_ascii_lowercase(), value.to_ascii_lowercase()))
            .collect();
        sorted_headers.sort_by(|(a, _), (b, _)| a.cmp(b));
        sorted_headers.hash(state)
    }
}

pub(crate) mod parser {
    use std::borrow::Cow;

    use crate::{
        parser::{
            alpha, digit, escaped, hex_digit, is_reserved, is_unreserved, reserved, take1, token,
            ttl, unreserved, ParserResult,
        },
        utils::has_unique_elements,
    };

    use super::*;
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

    #[inline]
    pub(super) fn is_user_unreserved(b: u8) -> bool {
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
    pub(super) fn is_password_special_char(b: u8) -> bool {
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
            |(user, password, _)| UserInfo { user, password },
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
            HostPort {
                host: host.to_string(),
                port,
            }
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
    pub(super) fn is_param_unreserved(b: u8) -> bool {
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

    fn uri_parameters(input: &[u8]) -> ParserResult<&[u8], Parameters> {
        context(
            "uri_parameters",
            map(many0(preceded(tag(";"), uri_parameter)), |parameters| {
                Parameters(
                    parameters
                        .into_iter()
                        .map(|(k, v)| {
                            (
                                k.into_owned(),
                                if v.is_empty() {
                                    None
                                } else {
                                    Some(v.into_owned())
                                },
                            )
                        })
                        .collect::<Vec<(String, Option<String>)>>(),
                )
            }),
        )(input)
    }

    #[inline]
    pub(super) fn is_hnv_unreserved(b: u8) -> bool {
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

    fn headers(input: &[u8]) -> ParserResult<&[u8], Headers> {
        context(
            "headers",
            map(
                pair(
                    preceded(tag("?"), header),
                    many0(preceded(tag("&"), header)),
                ),
                |(first_header, mut other_headers)| {
                    let mut headers = vec![first_header];
                    headers.append(&mut other_headers);
                    Headers(headers.into_iter().collect::<Vec<(String, String)>>())
                },
            ),
        )(input)
    }

    fn sip_uri(input: &[u8]) -> ParserResult<&[u8], Uri> {
        context(
            "sip",
            map(
                tuple((
                    tag_no_case("sip:"),
                    opt(userinfo),
                    hostport,
                    cut(verify(uri_parameters, |params| {
                        has_unique_elements(params.iter().map(|p| &p.0))
                    })),
                    opt(headers),
                )),
                |(_, userinfo, hostport, parameters, headers)| {
                    Uri::Sip(SipUri {
                        scheme: Scheme::SIP,
                        userinfo,
                        hostport,
                        parameters,
                        headers: headers.unwrap_or_default(),
                    })
                },
            ),
        )(input)
    }

    fn sips_uri(input: &[u8]) -> ParserResult<&[u8], Uri> {
        context(
            "sips_uri",
            map(
                tuple((
                    tag_no_case("sips:"),
                    opt(userinfo),
                    hostport,
                    uri_parameters,
                    opt(headers),
                )),
                |(_, userinfo, hostport, parameters, headers)| {
                    Uri::Sip(SipUri {
                        scheme: Scheme::SIPS,
                        userinfo,
                        hostport,
                        parameters,
                        headers: headers.unwrap_or_default(),
                    })
                },
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
                separated_pair(
                    verify(scheme, |s: &str| {
                        !(s.eq_ignore_ascii_case("sip") || s.eq_ignore_ascii_case("sips"))
                    }),
                    tag(":"),
                    opaque_part,
                ),
                |(scheme, opaque_part)| AbsoluteUri {
                    scheme: Scheme::Other(scheme.into_owned()),
                    opaque_part: opaque_part.into_owned(),
                    parameters: Parameters::default(),
                    headers: Headers::default(),
                },
            ),
        )(input)
    }

    pub(crate) fn uri(input: &[u8]) -> ParserResult<&[u8], Uri> {
        context(
            "uri",
            alt((sip_uri, sips_uri, map(absolute_uri, Uri::Absolute))),
        )(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_uri_parsing() {
        let uri = Uri::from_str("sip:alice@atlanta.com");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice@atlanta.com");

        let uri = Uri::from_str("sip:alice:secretword@atlanta.com;transport=tcp");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert_eq!(uri.get_password(), Some("secretword"));
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("transport"), Some("tcp"));
        assert!(uri.get_headers().is_empty());
        assert_eq!(
            uri.to_string(),
            "sip:alice:secretword@atlanta.com;transport=tcp"
        );

        let uri = Uri::from_str("sips:alice@atlanta.com?subject=project%20x&priority=urgent");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIPS);
        assert_eq!(uri.get_user(), Some("alice"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert_eq!(uri.get_headers().len(), 2);
        assert_eq!(uri.get_header("subject"), Some("project x"));
        assert_eq!(uri.get_header("priority"), Some("urgent"));
        assert_eq!(
            uri.to_string(),
            "sips:alice@atlanta.com?subject=project%20x&priority=urgent"
        );

        let uri = Uri::from_str("sip:+1-212-555-1212:1234@gateway.com;user=phone");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIP);
        assert_eq!(uri.get_user(), Some("+1-212-555-1212"));
        assert_eq!(uri.get_password(), Some("1234"));
        assert_eq!(uri.get_host(), "gateway.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("user"), Some("phone"));
        assert!(uri.get_headers().is_empty());
        assert_eq!(
            uri.to_string(),
            "sip:+1-212-555-1212:1234@gateway.com;user=phone"
        );

        let uri = Uri::from_str("sips:1212@gateway.com");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIPS);
        assert_eq!(uri.get_user(), Some("1212"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "gateway.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sips:1212@gateway.com");

        let uri = Uri::from_str("sip:alice@192.0.2.4");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "192.0.2.4");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice@192.0.2.4");

        let uri = Uri::from_str("sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIP);
        assert!(uri.get_user().is_none());
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("method"), Some("REGISTER"));
        assert_eq!(uri.get_headers().len(), 1);
        assert_eq!(uri.get_header("to"), Some("alice@atlanta.com"));
        assert_eq!(
            uri.to_string(),
            "sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com"
        );

        let uri = Uri::from_str("sip:alice;day=tuesday@atlanta.com");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIP);
        assert_eq!(uri.get_user(), Some("alice;day=tuesday"));
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice;day=tuesday@atlanta.com");

        // Check escaped char in password.
        let uri = Uri::from_str("sip:alice:secret%77ord@atlanta.com");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert_eq!(uri.get_password(), Some("secretword"));
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert!(uri.get_parameters().is_empty());
        assert!(uri.get_headers().is_empty());
        assert_eq!(uri.to_string(), "sip:alice:secretword@atlanta.com");

        // Check escaped chars in parameters.
        let uri = Uri::from_str("sip:alice:secretword@atlanta.com;%74ransport=t%63p");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIP);
        assert_eq!(uri.get_user(), Some("alice"));
        assert_eq!(uri.get_password(), Some("secretword"));
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("transport"), Some("tcp"));
        assert!(uri.get_headers().is_empty());
        assert_eq!(
            uri.to_string(),
            "sip:alice:secretword@atlanta.com;transport=tcp"
        );

        // Check escaped chars in headers.
        let uri = Uri::from_str("sip:atlanta.com;method=REGISTER?t%6f=al%69ce%40atlant%61.com");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.is_sip());
        assert!(!uri.is_secure());
        assert_eq!(uri.get_scheme(), Scheme::SIP);
        assert!(uri.get_user().is_none());
        assert!(uri.get_password().is_none());
        assert_eq!(uri.get_host(), "atlanta.com");
        assert!(uri.get_port().is_none());
        assert_eq!(uri.get_parameters().len(), 1);
        assert_eq!(uri.get_parameter("method"), Some("REGISTER"));
        assert_eq!(uri.get_headers().len(), 1);
        assert_eq!(uri.get_header("to"), Some("alice@atlanta.com"));
        assert_eq!(
            uri.to_string(),
            "sip:atlanta.com;method=REGISTER?to=alice%40atlanta.com"
        );
    }

    #[test]
    fn test_invalid_uri_parsing() {
        // Multiple parameters with the same name are not allowed.
        let uri = Uri::from_str("sip:alice@atlanta.com;transport=tcp;transport=udp");
        assert!(uri.is_err());

        // Invalid IPv4 address.
        let uri = Uri::from_str("sip:alice@1923.0.2.4");
        assert!(uri.is_err());
        let uri = Uri::from_str("sip:alice@192.0.329.18");
        assert!(uri.is_err());

        // Invalid multiple `@` characters.
        let uri = Uri::from_str("sip:alice@atlanta.com@gateway.com");
        assert!(uri.is_err());
    }

    #[test]
    fn test_uris_equal() {
        assert_eq!(
            Uri::from_str("sip:%61lice@atlanta.com;transport=TCP").unwrap(),
            Uri::from_str("sip:alice@AtLanTa.CoM;Transport=tcp").unwrap()
        );

        assert_eq!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com;newparam=5").unwrap()
        );
        assert_eq!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=on").unwrap()
        );
        assert_eq!(
            Uri::from_str("sip:carol@chicago.com;newparam=5").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=on").unwrap()
        );

        assert_eq!(
            Uri::from_str("sip:biloxi.com;transport=tcp;method=REGISTER?to=sip:bob%40biloxi.com")
                .unwrap(),
            Uri::from_str("sip:biloxi.com;method=REGISTER;transport=tcp?to=sip:bob%40biloxi.com")
                .unwrap()
        );

        assert_eq!(
            Uri::from_str("sip:alice@atlanta.com?subject=project%20x&priority=urgent").unwrap(),
            Uri::from_str("sip:alice@atlanta.com?priority=urgent&subject=project%20x").unwrap()
        );
    }

    #[test]
    fn test_uris_not_equal() {
        assert_ne!(
            Uri::from_str("SIP:ALICE@AtLanTa.CoM;Transport=udp").unwrap(),
            Uri::from_str("sip:alice@AtLanTa.CoM;Transport=UDP").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:bob@biloxi.com").unwrap(),
            Uri::from_str("sip:bob@biloxi.com:5060").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:bob@biloxi.com").unwrap(),
            Uri::from_str("sip:bob@biloxi.com;transport=udp").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:bob@biloxi.com").unwrap(),
            Uri::from_str("sip:bob@biloxi.com:6000;transport=tcp").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com?Subject=next%20meeting").unwrap()
        );

        assert_ne!(
            Uri::from_str("sip:bob@phone21.boxesbybob.com").unwrap(),
            Uri::from_str("sip:bob@192.0.2.4").unwrap()
        );
    }

    #[test]
    fn test_uris_intransitivity() {
        assert_eq!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=on").unwrap()
        );
        assert_eq!(
            Uri::from_str("sip:carol@chicago.com").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=off").unwrap()
        );
        assert_ne!(
            Uri::from_str("sip:carol@chicago.com;security=on").unwrap(),
            Uri::from_str("sip:carol@chicago.com;security=off").unwrap()
        );
    }
}
