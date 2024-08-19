//! Parsing and generation of a SIP URI.

use crate::{
    Host, IntoHost, IntoPort, IntoSpecificString, Method, PasswordString, SipError, Transport,
    UriHeader, UriHeaderNameString, UriHeaderValueString, UriHeaders, UriParameter,
    UriParameterString, UriParameters, UriScheme, UserInfo, UserString, UserType,
};

/// Representation of a SIP URI.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct SipUri {
    scheme: UriScheme,
    userinfo: Option<UserInfo>,
    host: Host,
    port: Option<u16>,
    parameters: UriParameters,
    headers: UriHeaders,
}

impl SipUri {
    pub(crate) fn new(
        scheme: UriScheme,
        userinfo: Option<UserInfo>,
        host: Host,
        port: Option<u16>,
        parameters: UriParameters,
        headers: UriHeaders,
    ) -> Self {
        Self {
            scheme,
            userinfo,
            host,
            port,
            parameters,
            headers,
        }
    }

    /// Get a reference to the `UriScheme` of the sip uri.
    pub fn scheme(&self) -> &UriScheme {
        &self.scheme
    }

    /// Get a reference to the `UserInfo` of the sip uri.
    pub fn userinfo(&self) -> Option<&UserInfo> {
        self.userinfo.as_ref()
    }

    /// Get a reference to the `Host` of the sip uri.
    pub fn host(&self) -> &Host {
        &self.host
    }

    /// Get the port of the sip uri.
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Get a reference to the `UriParameters` of the sip uri.
    pub fn parameters(&self) -> &UriParameters {
        &self.parameters
    }

    /// Get a reference to the `UriHeaders` of the sip uri.
    pub fn headers(&self) -> &UriHeaders {
        &self.headers
    }

    /// Get a `SipUri` builder.
    pub fn builder() -> SipUriBuilder {
        SipUriBuilder::default()
    }
}

impl std::fmt::Display for SipUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}{}{}{}{}{}{}{}{}",
            self.scheme,
            if let Some(userinfo) = &self.userinfo {
                format!("{}", userinfo)
            } else {
                "".to_owned()
            },
            if self.userinfo.is_some() { "@" } else { "" },
            self.host,
            if self.port.is_some() { ":" } else { "" },
            self.port.map(|p| p.to_string()).unwrap_or_default(),
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters,
            if self.headers.is_empty() { "" } else { "?" },
            self.headers
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct SipUriBuilder {
    scheme: UriScheme,
    user: Option<UserString>,
    password: Option<PasswordString>,
    host: Host,
    port: Option<u16>,
    parameters: UriParameters,
    headers: UriHeaders,
}

impl SipUriBuilder {
    pub fn scheme(&mut self, scheme: UriScheme) -> Result<&mut Self, SipError> {
        match scheme {
            UriScheme::Sip | UriScheme::Sips => {
                self.scheme = scheme;
                Ok(self)
            }
            _ => Err(SipError::InvalidUriScheme(scheme.to_string())),
        }
    }

    pub fn user<U: Into<IntoSpecificString<UserString>>>(
        &mut self,
        user: U,
    ) -> Result<&mut Self, SipError> {
        let user = user.into();
        self.user = Some(user.try_into()?);
        Ok(self)
    }

    pub fn password<P: Into<IntoSpecificString<PasswordString>>>(
        &mut self,
        password: P,
    ) -> Result<&mut Self, SipError> {
        let password = password.into();
        self.password = Some(password.try_into()?);
        Ok(self)
    }

    pub fn host<H: Into<IntoHost>>(&mut self, host: H) -> Result<&mut Self, SipError> {
        let host = host.into();
        self.host = host.try_into()?;
        Ok(self)
    }

    pub fn port<P: Into<IntoPort>>(&mut self, port: P) -> &mut Self {
        let port = port.into();
        self.port = port.into();
        self
    }

    pub fn transport_parameter(&mut self, transport: Transport) -> &mut Self {
        self.parameters
            .add_parameter(UriParameter::Transport(transport));
        self
    }

    pub fn user_parameter(&mut self, user: UserType) -> &mut Self {
        self.parameters.add_parameter(UriParameter::User(user));
        self
    }

    pub fn method_parameter(&mut self, method: Method) -> &mut Self {
        self.parameters.add_parameter(UriParameter::Method(method));
        self
    }

    pub fn ttl_parameter(&mut self, ttl: u8) -> &mut Self {
        self.parameters.add_parameter(UriParameter::Ttl(ttl));
        self
    }

    pub fn maddr_parameter<H: Into<IntoHost>>(&mut self, maddr: H) -> Result<&mut Self, SipError> {
        let maddr = maddr.into();
        self.parameters
            .add_parameter(UriParameter::MAddr(maddr.try_into()?));
        Ok(self)
    }

    pub fn parameter<P: Into<IntoSpecificString<UriParameterString>>>(
        &mut self,
        name: P,
        value: Option<P>,
    ) -> Result<&mut Self, SipError> {
        let name = name.into();
        let value = value.map(Into::into);
        let name: UriParameterString = name.try_into()?;
        let parameter = UriParameter::try_from(
            format!(
                "{}={}",
                name,
                match value {
                    Some(value) => {
                        let value: UriParameterString = value.try_into()?;
                        value.to_string()
                    }
                    None => "".to_string(),
                }
            )
            .as_str(),
        )?;
        self.parameters.add_parameter(parameter);
        Ok(self)
    }

    pub fn clear_parameters(&mut self) -> &mut Self {
        self.parameters.clear();
        self
    }

    pub fn header<
        N: Into<IntoSpecificString<UriHeaderNameString>>,
        V: Into<IntoSpecificString<UriHeaderValueString>>,
    >(
        &mut self,
        name: N,
        value: V,
    ) -> Result<&mut Self, SipError> {
        let name = name.into();
        let value = value.into();
        self.headers
            .push(UriHeader::new(name.try_into()?, value.try_into()?));
        Ok(self)
    }

    pub fn clear_headers(&mut self) -> &mut Self {
        self.headers.clear();
        self
    }

    pub fn build(&self) -> SipUri {
        let mut uri = SipUri {
            scheme: Clone::clone(&self.scheme),
            host: Clone::clone(&self.host),
            port: self.port,
            parameters: Clone::clone(&self.parameters),
            headers: Clone::clone(&self.headers),
            ..Default::default()
        };
        if let Some(user) = &self.user {
            uri.userinfo = Some(UserInfo::new(
                Clone::clone(user),
                Clone::clone(&self.password),
            ));
        }
        uri
    }
}

pub(crate) mod parser {
    use crate::parser::ParserResult;
    use crate::uris::host::parser::hostport;
    use crate::uris::uri_header::parser::headers;
    use crate::uris::uri_parameter::parser::uri_parameters;
    use crate::uris::user_info::parser::userinfo;
    use crate::{SipUri, Uri, UriScheme};
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        combinator::{cut, map, opt},
        error::context,
        sequence::{pair, tuple},
    };

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
                        uri_parameters,
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
}

#[cfg(test)]
mod tests {
    use crate::common::wrapped_string::WrappedString;
    use crate::{
        GenericParameter, Host, Method, Transport, UriHeader, UriHeaderNameString,
        UriHeaderValueString, UriParameter, UriParameterString, UriSchemeString, UserType,
    };
    use crate::{HostnameString, SipUri, UriScheme};
    use claims::{assert_err, assert_ok};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_valid_sip_uri_builder() {
        let uri = SipUri::builder().build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert_eq!(uri.userinfo(), None);
        assert_eq!(uri.host(), &Host::Name(HostnameString::new("localhost")));
        assert_eq!(uri.port(), None);
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(uri.to_string(), "sip:localhost");
    }

    #[test]
    fn test_valid_sip_uri_with_hostname_builder() {
        let uri = SipUri::builder()
            .host("atlanta.com")
            .unwrap()
            .port(5060)
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert_eq!(uri.userinfo(), None);
        assert_eq!(uri.host(), &Host::Name(HostnameString::new("atlanta.com")));
        assert_eq!(uri.port(), Some(5060));
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(uri.to_string(), "sip:atlanta.com:5060");
    }

    #[test]
    fn test_valid_sip_uri_with_ipv4_builder() {
        let uri = SipUri::builder()
            .host(Ipv4Addr::new(192, 168, 0, 1))
            .unwrap()
            .port(Some(1234))
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert_eq!(uri.userinfo(), None);
        assert_eq!(
            uri.host(),
            &Host::Ip(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)))
        );
        assert_eq!(uri.port(), Some(1234));
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(uri.to_string(), "sip:192.168.0.1:1234");
    }

    #[test]
    fn test_valid_sip_uri_with_ipv6_builder() {
        let uri = SipUri::builder()
            .host(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x8190, 0x3426))
            .unwrap()
            .port(8012)
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert_eq!(uri.userinfo(), None);
        assert_eq!(
            uri.host(),
            &Host::Ip(IpAddr::V6(Ipv6Addr::new(
                0, 0, 0, 0, 0, 0xffff, 0x8190, 0x3426
            )))
        );
        assert_eq!(uri.port(), Some(8012));
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(uri.to_string(), "sip:[::ffff:129.144.52.38]:8012");
    }

    #[test]
    fn test_valid_sips_uri_with_ip_builder() {
        let uri = SipUri::builder()
            .scheme(UriScheme::Sips)
            .unwrap()
            .host(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)))
            .unwrap()
            .port(Some(1234))
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sips);
        assert_eq!(uri.userinfo(), None);
        assert_eq!(
            uri.host(),
            &Host::Ip(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)))
        );
        assert_eq!(uri.port(), Some(1234));
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(uri.to_string(), "sips:192.168.0.1:1234");
    }

    #[test]
    fn test_valid_sip_uri_with_user_builder() {
        let uri = SipUri::builder()
            .user("alice")
            .unwrap()
            .host("atlanta.com")
            .unwrap()
            .port(None)
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert!(uri.userinfo().is_some());
        assert_eq!(uri.userinfo().unwrap().user(), "alice");
        assert_eq!(uri.userinfo().unwrap().password(), None);
        assert_eq!(uri.host(), &Host::Name(HostnameString::new("atlanta.com")));
        assert_eq!(uri.port(), None);
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(uri.to_string(), "sip:alice@atlanta.com");
    }

    #[test]
    fn test_valid_sip_uri_with_user_and_password_builder() {
        let uri = SipUri::builder()
            .user("alice")
            .unwrap()
            .password("secret word")
            .unwrap()
            .host("atlanta.com")
            .unwrap()
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert!(uri.userinfo().is_some());
        assert_eq!(uri.userinfo().unwrap().user(), "alice");
        assert_eq!(uri.userinfo().unwrap().password(), Some("secret word"));
        assert_eq!(uri.host(), &Host::Name(HostnameString::new("atlanta.com")));
        assert_eq!(uri.port(), None);
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(uri.to_string(), "sip:alice:secret%20word@atlanta.com");
    }

    #[test]
    fn test_valid_sip_uri_with_headers_builder() {
        let uri = SipUri::builder()
            .user("alice")
            .unwrap()
            .host("atlanta.com")
            .unwrap()
            .header("subject", "project")
            .unwrap()
            .header("priority", "urgent")
            .unwrap()
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert!(uri.userinfo().is_some());
        assert_eq!(uri.userinfo().unwrap().user(), "alice");
        assert_eq!(uri.userinfo().unwrap().password(), None);
        assert_eq!(uri.host(), &Host::Name(HostnameString::new("atlanta.com")));
        assert_eq!(uri.port(), None);
        assert_eq!(uri.headers().len(), 2);
        let mut headers_it = uri.headers().iter();
        let header = headers_it.next().unwrap();
        assert_eq!(
            header,
            &UriHeader::new(
                UriHeaderNameString::new("subject"),
                UriHeaderValueString::new("project")
            )
        );
        let header = headers_it.next().unwrap();
        assert_eq!(
            header,
            &UriHeader::new(
                UriHeaderNameString::new("priority"),
                UriHeaderValueString::new("urgent")
            )
        );
        assert_eq!(
            uri.to_string(),
            "sip:alice@atlanta.com?subject=project&priority=urgent"
        );
    }

    #[test]
    fn test_valid_sip_uri_with_header_with_empty_value_builder() {
        let uri = SipUri::builder()
            .user("alice")
            .unwrap()
            .host("atlanta.com")
            .unwrap()
            .header("subject", "")
            .unwrap()
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert!(uri.userinfo().is_some());
        assert_eq!(uri.userinfo().unwrap().user(), "alice");
        assert_eq!(uri.userinfo().unwrap().password(), None);
        assert_eq!(uri.host(), &Host::Name(HostnameString::new("atlanta.com")));
        assert_eq!(uri.port(), None);
        assert_eq!(uri.headers().len(), 1);
        assert_eq!(
            uri.headers.iter().next().unwrap(),
            &UriHeader::new(
                UriHeaderNameString::new("subject"),
                UriHeaderValueString::new("")
            )
        );
        assert_eq!(uri.to_string(), "sip:alice@atlanta.com?subject=");
    }

    #[test]
    fn test_valid_sip_uri_with_parameters_builder() {
        let uri = SipUri::builder()
            .user("+33612345678")
            .unwrap()
            .host("atlanta.com")
            .unwrap()
            .transport_parameter(Transport::Tcp)
            .user_parameter(UserType::Phone)
            .method_parameter(Method::Invite)
            .ttl_parameter(25)
            .maddr_parameter(Ipv4Addr::new(192, 168, 0, 1))
            .unwrap()
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert!(uri.userinfo().is_some());
        assert_eq!(uri.userinfo().unwrap().user(), "+33612345678");
        assert_eq!(uri.userinfo().unwrap().password(), None);
        assert_eq!(uri.host(), &Host::Name(HostnameString::new("atlanta.com")));
        assert_eq!(uri.port(), None);
        assert_eq!(uri.parameters().len(), 5);
        let mut parameters_it = uri.parameters().iter();
        let parameter = parameters_it.next().unwrap();
        assert_eq!(parameter, &UriParameter::Transport(Transport::Tcp));
        let parameter = parameters_it.next().unwrap();
        assert_eq!(parameter, &UriParameter::User(UserType::Phone));
        let parameter = parameters_it.next().unwrap();
        assert_eq!(parameter, &UriParameter::Method(Method::Invite));
        let parameter = parameters_it.next().unwrap();
        assert_eq!(parameter, &UriParameter::Ttl(25));
        let parameter = parameters_it.next().unwrap();
        assert_eq!(
            parameter,
            &UriParameter::MAddr(Host::Ip(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1))))
        );
        assert_eq!(parameters_it.next(), None);
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(
            uri.to_string(),
            "sip:+33612345678@atlanta.com;transport=tcp;user=phone;method=INVITE;ttl=25;maddr=192.168.0.1"
        );
    }

    #[test]
    fn test_valid_sip_uri_with_custom_parameters_builder() {
        let uri = SipUri::builder()
            .user("alice")
            .unwrap()
            .host("atlanta.com")
            .unwrap()
            .parameter("myparam1", Some("foo"))
            .unwrap()
            .parameter("myparam2", Some("bar"))
            .unwrap()
            .parameter("transport", Some("TCP"))
            .unwrap()
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert!(uri.userinfo().is_some());
        assert_eq!(uri.userinfo().unwrap().user(), "alice");
        assert_eq!(uri.userinfo().unwrap().password(), None);
        assert_eq!(uri.host(), &Host::Name(HostnameString::new("atlanta.com")));
        assert_eq!(uri.port(), None);
        assert_eq!(uri.parameters().len(), 3);
        let mut parameters_it = uri.parameters().iter();
        let parameter = parameters_it.next().unwrap();
        assert_eq!(
            parameter,
            &UriParameter::Other(GenericParameter::new(
                UriParameterString::new("myparam1"),
                Some(WrappedString::NotWrapped(UriParameterString::new("foo")))
            ))
        );
        let parameter = parameters_it.next().unwrap();
        assert_eq!(
            parameter,
            &UriParameter::Other(GenericParameter::new(
                UriParameterString::new("myparam2"),
                Some(WrappedString::NotWrapped(UriParameterString::new("bar")))
            ))
        );
        let parameter = parameters_it.next().unwrap();
        assert_eq!(parameter, &UriParameter::Transport(Transport::Tcp));
        assert_eq!(parameters_it.next(), None);
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(
            uri.to_string(),
            "sip:alice@atlanta.com;myparam1=foo;myparam2=bar;transport=tcp"
        );
    }

    #[test]
    fn test_valid_sip_uri_with_duplicated_parameter_builder() {
        let uri = SipUri::builder()
            .user("alice")
            .unwrap()
            .host("atlanta.com")
            .unwrap()
            .user_parameter(UserType::Ip)
            .parameter("user", Some("phone"))
            .unwrap()
            .build();
        assert_eq!(uri.scheme(), &UriScheme::Sip);
        assert!(uri.userinfo().is_some());
        assert_eq!(uri.userinfo().unwrap().user(), "alice");
        assert_eq!(uri.userinfo().unwrap().password(), None);
        assert_eq!(uri.host(), &Host::Name(HostnameString::new("atlanta.com")));
        assert_eq!(uri.port(), None);
        assert_eq!(uri.parameters().len(), 1);
        let mut parameters_it = uri.parameters().iter();
        let parameter = parameters_it.next().unwrap();
        assert_eq!(parameter, &UriParameter::User(UserType::Phone));
        assert_eq!(parameters_it.next(), None);
        assert_eq!(uri.headers().len(), 0);
        assert_eq!(uri.to_string(), "sip:alice@atlanta.com;user=phone");
    }

    #[test]
    fn test_invalid_sip_uri_builder_invalid_scheme() {
        let scheme = UriSchemeString::try_from("http");
        assert_ok!(&scheme);
        if let Ok(scheme) = scheme {
            assert_err!(SipUri::builder().scheme(UriScheme::Other(scheme)));
        }
    }

    #[test]
    fn test_invalid_sip_uri_builder_invalid_hostname_with_trailing_dash() {
        assert_err!(SipUri::builder().host("atlanta-.com"));
    }

    #[test]
    fn test_invalid_sip_uri_builder_invalid_hostname_with_utf8_char() {
        assert_err!(SipUri::builder().host("électricité.fr"));
    }

    #[test]
    fn test_invalid_sip_uri_builder_empty_hostname() {
        assert_err!(SipUri::builder().host(""));
    }

    #[test]
    fn test_invalid_sip_uri_builder_invalid_ip_addr() {
        assert_err!(SipUri::builder().host("1928.68.1983.0"));
    }

    #[test]
    fn test_invalid_sip_uri_builder_empty_password() {
        assert_err!(SipUri::builder().user(""));
    }

    #[test]
    fn test_invalid_sip_uri_builder_with_empty_header_name() {
        assert_err!(SipUri::builder().header("", ""));
    }
}
