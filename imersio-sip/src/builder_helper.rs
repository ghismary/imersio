use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{
    Host, HostnameString, Method, OpaquePartString, PasswordString, SipError, UriHeaderNameString,
    UriHeaderValueString, UriParameterString, UriScheme, UserString,
};

/// Helper enum to build `UriScheme` values.
#[derive(Debug)]
pub enum IntoUriScheme {
    /// Input as a string.
    String(String),
    /// Input as a `UriScheme`.
    UriScheme(UriScheme),
}

impl From<&str> for IntoUriScheme {
    fn from(value: &str) -> Self {
        IntoUriScheme::String(value.to_string())
    }
}

impl From<String> for IntoUriScheme {
    fn from(value: String) -> Self {
        IntoUriScheme::String(value)
    }
}

impl From<UriScheme> for IntoUriScheme {
    fn from(value: UriScheme) -> Self {
        IntoUriScheme::UriScheme(value)
    }
}

impl TryFrom<IntoUriScheme> for UriScheme {
    type Error = SipError;

    fn try_from(value: IntoUriScheme) -> Result<Self, Self::Error> {
        Ok(match value {
            IntoUriScheme::String(value) => UriScheme::try_from(value.as_str())?,
            IntoUriScheme::UriScheme(value) => value,
        })
    }
}

/// Helper enum to build `Host` values.
#[derive(Debug)]
pub enum IntoHost {
    /// Input as a string.
    String(String),
    /// Input as an IpAddr.
    IpAddr(IpAddr),
    /// Input as a `HostnameString`.
    HostnameString(HostnameString),
    /// Input as a `Host`.
    Host(Host),
}

impl From<&str> for IntoHost {
    fn from(value: &str) -> Self {
        IntoHost::String(value.to_string())
    }
}

impl From<String> for IntoHost {
    fn from(value: String) -> Self {
        IntoHost::String(value)
    }
}

impl From<IpAddr> for IntoHost {
    fn from(value: IpAddr) -> Self {
        IntoHost::IpAddr(value)
    }
}

impl From<Ipv4Addr> for IntoHost {
    fn from(value: Ipv4Addr) -> Self {
        IntoHost::IpAddr(IpAddr::V4(value))
    }
}

impl From<Ipv6Addr> for IntoHost {
    fn from(value: Ipv6Addr) -> Self {
        IntoHost::IpAddr(IpAddr::V6(value))
    }
}

impl From<HostnameString> for IntoHost {
    fn from(value: HostnameString) -> Self {
        IntoHost::HostnameString(value)
    }
}

impl TryFrom<IntoHost> for Host {
    type Error = SipError;

    fn try_from(value: IntoHost) -> Result<Self, Self::Error> {
        Ok(match value {
            IntoHost::String(value) => Host::try_from(value.as_str())?,
            IntoHost::IpAddr(value) => Host::Ip(value),
            IntoHost::HostnameString(value) => Host::Name(value),
            IntoHost::Host(value) => value,
        })
    }
}

/// Helper struct to build u16 values for the port of a SIP URI.
#[derive(Debug)]
pub struct IntoPort(Option<u16>);

impl From<u16> for IntoPort {
    fn from(value: u16) -> Self {
        IntoPort(Some(value))
    }
}

impl From<Option<u16>> for IntoPort {
    fn from(value: Option<u16>) -> Self {
        IntoPort(value)
    }
}

impl From<IntoPort> for Option<u16> {
    fn from(value: IntoPort) -> Self {
        value.0
    }
}

/// Helper enum to build `Host` values.
#[derive(Debug)]
pub enum IntoMethod {
    /// Input as a string.
    String(String),
    /// Input as a `Method`.
    Method(Method),
}

impl From<&str> for IntoMethod {
    fn from(value: &str) -> Self {
        IntoMethod::String(value.to_string())
    }
}

impl From<String> for IntoMethod {
    fn from(value: String) -> Self {
        IntoMethod::String(value)
    }
}

impl From<Method> for IntoMethod {
    fn from(value: Method) -> Self {
        IntoMethod::Method(value)
    }
}

impl TryFrom<IntoMethod> for Method {
    type Error = SipError;

    fn try_from(value: IntoMethod) -> Result<Self, Self::Error> {
        Ok(match value {
            IntoMethod::String(value) => Method::try_from(value.to_ascii_uppercase().as_str())?,
            IntoMethod::Method(value) => value,
        })
    }
}

/// Helper enum to build specific string values such as `UserString`.
#[derive(Debug)]
pub enum IntoSpecificString<T> {
    /// Input as a string.
    String(String),
    /// Input as a specific string.
    SpecificString(T),
}

impl<T> From<&str> for IntoSpecificString<T> {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl<T> From<String> for IntoSpecificString<T> {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<UserString> for IntoSpecificString<UserString> {
    fn from(value: UserString) -> Self {
        Self::SpecificString(value)
    }
}

impl TryFrom<IntoSpecificString<UserString>> for UserString {
    type Error = SipError;
    fn try_from(value: IntoSpecificString<UserString>) -> Result<Self, Self::Error> {
        Ok(match value {
            IntoSpecificString::String(value) => UserString::try_from(value.as_str())?,
            IntoSpecificString::SpecificString(value) => value,
        })
    }
}

impl From<PasswordString> for IntoSpecificString<PasswordString> {
    fn from(value: PasswordString) -> Self {
        Self::SpecificString(value)
    }
}

impl TryFrom<IntoSpecificString<PasswordString>> for PasswordString {
    type Error = SipError;
    fn try_from(value: IntoSpecificString<PasswordString>) -> Result<Self, Self::Error> {
        Ok(match value {
            IntoSpecificString::String(value) => PasswordString::try_from(value.as_str())?,
            IntoSpecificString::SpecificString(value) => value,
        })
    }
}

impl From<UriHeaderNameString> for IntoSpecificString<UriHeaderNameString> {
    fn from(value: UriHeaderNameString) -> Self {
        Self::SpecificString(value)
    }
}

impl TryFrom<IntoSpecificString<UriHeaderNameString>> for UriHeaderNameString {
    type Error = SipError;
    fn try_from(value: IntoSpecificString<UriHeaderNameString>) -> Result<Self, Self::Error> {
        Ok(match value {
            IntoSpecificString::String(value) => UriHeaderNameString::try_from(value.as_str())?,
            IntoSpecificString::SpecificString(value) => value,
        })
    }
}

impl From<UriHeaderValueString> for IntoSpecificString<UriHeaderValueString> {
    fn from(value: UriHeaderValueString) -> Self {
        Self::SpecificString(value)
    }
}

impl TryFrom<IntoSpecificString<UriHeaderValueString>> for UriHeaderValueString {
    type Error = SipError;
    fn try_from(value: IntoSpecificString<UriHeaderValueString>) -> Result<Self, Self::Error> {
        Ok(match value {
            IntoSpecificString::String(value) => UriHeaderValueString::try_from(value.as_str())?,
            IntoSpecificString::SpecificString(value) => value,
        })
    }
}

impl From<UriParameterString> for IntoSpecificString<UriParameterString> {
    fn from(value: UriParameterString) -> Self {
        Self::SpecificString(value)
    }
}

impl TryFrom<IntoSpecificString<UriParameterString>> for UriParameterString {
    type Error = SipError;
    fn try_from(value: IntoSpecificString<UriParameterString>) -> Result<Self, Self::Error> {
        Ok(match value {
            IntoSpecificString::String(value) => UriParameterString::try_from(value.as_str())?,
            IntoSpecificString::SpecificString(value) => value,
        })
    }
}

impl From<OpaquePartString> for IntoSpecificString<OpaquePartString> {
    fn from(value: OpaquePartString) -> Self {
        Self::SpecificString(value)
    }
}

impl TryFrom<IntoSpecificString<OpaquePartString>> for OpaquePartString {
    type Error = SipError;
    fn try_from(value: IntoSpecificString<OpaquePartString>) -> Result<Self, Self::Error> {
        Ok(match value {
            IntoSpecificString::String(value) => OpaquePartString::try_from(value.as_str())?,
            IntoSpecificString::SpecificString(value) => value,
        })
    }
}
