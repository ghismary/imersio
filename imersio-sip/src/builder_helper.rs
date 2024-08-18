use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{
    Host, HostnameString, PasswordString, UriHeaderNameString, UriHeaderValueString, UserString,
};

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

/// Helper enum to build `UserString` values.
#[derive(Debug)]
pub enum IntoUserString {
    /// Input as a string.
    String(String),
    /// Input as a `UserString`.
    UserString(UserString),
}

impl From<&str> for IntoUserString {
    fn from(value: &str) -> Self {
        IntoUserString::String(value.to_string())
    }
}

impl From<String> for IntoUserString {
    fn from(value: String) -> Self {
        IntoUserString::String(value)
    }
}

impl From<UserString> for IntoUserString {
    fn from(value: UserString) -> Self {
        IntoUserString::UserString(value)
    }
}

/// Helper enum to build `PasswordString` values.
#[derive(Debug)]
pub enum IntoPasswordString {
    /// Input as a string.
    String(String),
    /// Input as a `PasswordString`.
    PasswordString(PasswordString),
}

impl From<&str> for IntoPasswordString {
    fn from(value: &str) -> Self {
        IntoPasswordString::String(value.to_string())
    }
}

impl From<String> for IntoPasswordString {
    fn from(value: String) -> Self {
        IntoPasswordString::String(value)
    }
}

impl From<PasswordString> for IntoPasswordString {
    fn from(value: PasswordString) -> Self {
        IntoPasswordString::PasswordString(value)
    }
}

/// Helper enum to build `UriHeaderNameString` values.
#[derive(Debug)]
pub enum IntoUriHeaderNameString {
    /// Input as a string.
    String(String),
    /// Input as a `UriHeaderString`.
    UriHeaderNameString(UriHeaderNameString),
}

impl From<&str> for IntoUriHeaderNameString {
    fn from(value: &str) -> Self {
        IntoUriHeaderNameString::String(value.to_string())
    }
}

impl From<String> for IntoUriHeaderNameString {
    fn from(value: String) -> Self {
        IntoUriHeaderNameString::String(value)
    }
}

impl From<UriHeaderNameString> for IntoUriHeaderNameString {
    fn from(value: UriHeaderNameString) -> Self {
        IntoUriHeaderNameString::UriHeaderNameString(value)
    }
}

/// Helper enum to build `UriHeaderValueString` values.
#[derive(Debug)]
pub enum IntoUriHeaderValueString {
    /// Input as a string.
    String(String),
    /// Input as a `UriHeaderValueString`.
    UriHeaderValueString(UriHeaderValueString),
}

impl From<&str> for IntoUriHeaderValueString {
    fn from(value: &str) -> Self {
        IntoUriHeaderValueString::String(value.to_string())
    }
}

impl From<String> for IntoUriHeaderValueString {
    fn from(value: String) -> Self {
        IntoUriHeaderValueString::String(value)
    }
}

impl From<UriHeaderValueString> for IntoUriHeaderValueString {
    fn from(value: UriHeaderValueString) -> Self {
        IntoUriHeaderValueString::UriHeaderValueString(value)
    }
}
