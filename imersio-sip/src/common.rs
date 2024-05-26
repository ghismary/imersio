//! TODO

use std::{cmp::Ordering, hash::Hash};

use crate::Uri;

/// Representation of a generic parameter.
#[derive(Clone, Debug, Eq)]
pub struct GenericParameter {
    key: String,
    value: Option<String>,
}

impl GenericParameter {
    /// Create a `GenericParam`.
    pub fn new<S: Into<String>>(key: S, value: Option<S>) -> Self {
        Self {
            key: key.into(),
            value: value.map(Into::into),
        }
    }

    /// Get the key of the `GenericParam`.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the value of the `GenericParam`.
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

impl std::fmt::Display for GenericParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.key,
            if self.value.is_some() { "=" } else { "" },
            self.value.as_deref().unwrap_or_default()
        )
    }
}

impl PartialEq<GenericParameter> for GenericParameter {
    fn eq(&self, other: &GenericParameter) -> bool {
        self.key().eq_ignore_ascii_case(other.key())
            && self.value().map(|v| v.to_ascii_lowercase())
                == other.value().map(|v| v.to_ascii_lowercase())
    }
}

impl PartialEq<&GenericParameter> for GenericParameter {
    fn eq(&self, other: &&GenericParameter) -> bool {
        self == *other
    }
}

impl PartialEq<GenericParameter> for &GenericParameter {
    fn eq(&self, other: &GenericParameter) -> bool {
        *self == other
    }
}

impl Hash for GenericParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        self.value().map(|v| v.to_ascii_lowercase()).hash(state);
    }
}

#[derive(Clone, Debug, Eq)]
pub enum AcceptParameter {
    Q(String),
    Other(GenericParameter),
}

impl AcceptParameter {
    pub(crate) fn new<S: Into<String>>(key: S, value: Option<S>) -> Self {
        let key: String = key.into();
        let value: Option<String> = value.map(Into::into);
        match (key.as_str(), &value) {
            ("q", Some(value)) => Self::Q(value.to_string()),
            _ => Self::Other(GenericParameter::new(key, value)),
        }
    }

    pub fn q(&self) -> Option<f32> {
        match self {
            Self::Q(value) => value.parse().ok(),
            _ => None,
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Self::Q(_) => "q",
            Self::Other(value) => value.key(),
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Q(value) => Some(value),
            Self::Other(value) => value.value(),
        }
    }
}

impl std::fmt::Display for AcceptParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Q(value) => write!(f, "q={value}"),
            Self::Other(value) => write!(
                f,
                "{}{}{}",
                value.key(),
                if value.value().is_some() { "=" } else { "" },
                value.value().unwrap_or_default()
            ),
        }
    }
}

impl PartialEq<AcceptParameter> for AcceptParameter {
    fn eq(&self, other: &AcceptParameter) -> bool {
        match (self, other) {
            (Self::Q(a), Self::Q(b)) => a == b,
            (Self::Other(a), Self::Other(b)) => {
                a.key().eq_ignore_ascii_case(b.key())
                    && a.value().map(|v| v.to_ascii_lowercase())
                        == b.value().map(|v| v.to_ascii_lowercase())
            }
            _ => false,
        }
    }
}

impl PartialEq<&AcceptParameter> for AcceptParameter {
    fn eq(&self, other: &&AcceptParameter) -> bool {
        self == *other
    }
}

impl PartialEq<AcceptParameter> for &AcceptParameter {
    fn eq(&self, other: &AcceptParameter) -> bool {
        *self == other
    }
}

impl Hash for AcceptParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        self.value().map(|v| v.to_ascii_lowercase()).hash(state);
    }
}

impl PartialOrd for AcceptParameter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AcceptParameter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
    }
}

impl From<GenericParameter> for AcceptParameter {
    fn from(value: GenericParameter) -> Self {
        Self::Other(GenericParameter::new(value.key(), value.value()))
    }
}

#[derive(Clone, Debug, Eq)]
pub enum Algorithm {
    Md5,
    Md5Sess,
    Other(String),
}

impl Algorithm {
    pub(crate) fn new<S: Into<String>>(algo: S) -> Self {
        let algo: String = algo.into();
        match algo.to_ascii_lowercase().as_str() {
            "md5" => Self::Md5,
            "md5-sess" => Self::Md5Sess,
            _ => Self::Other(algo),
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Md5 => "MD5",
            Self::Md5Sess => "MD5-Sess",
            Self::Other(value) => value,
        }
    }
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq<Algorithm> for Algorithm {
    fn eq(&self, other: &Algorithm) -> bool {
        match (self, other) {
            (Self::Md5, Self::Md5) | (Self::Md5Sess, Self::Md5Sess) => true,
            (Self::Other(a), Self::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialEq<&Algorithm> for Algorithm {
    fn eq(&self, other: &&Algorithm) -> bool {
        self == *other
    }
}

impl PartialEq<Algorithm> for &Algorithm {
    fn eq(&self, other: &Algorithm) -> bool {
        *self == other
    }
}

impl Hash for Algorithm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().to_ascii_lowercase().hash(state);
    }
}

#[derive(Clone, Debug, Eq)]
pub enum MessageQop {
    Auth,
    AuthInt,
    Other(String),
}

impl MessageQop {
    pub(crate) fn new<S: Into<String>>(qop: S) -> Self {
        let qop: String = qop.into();
        match qop.to_ascii_lowercase().as_str() {
            "auth" => Self::Auth,
            "auth-int" => Self::AuthInt,
            _ => Self::Other(qop),
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Auth => "auth",
            Self::AuthInt => "auth-int",
            Self::Other(value) => value,
        }
    }
}

impl std::fmt::Display for MessageQop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq<MessageQop> for MessageQop {
    fn eq(&self, other: &MessageQop) -> bool {
        match (self, other) {
            (Self::Auth, Self::Auth) | (Self::AuthInt, Self::AuthInt) => true,
            (Self::Other(a), Self::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialEq<&MessageQop> for MessageQop {
    fn eq(&self, other: &&MessageQop) -> bool {
        self == *other
    }
}

impl PartialEq<MessageQop> for &MessageQop {
    fn eq(&self, other: &MessageQop) -> bool {
        *self == other
    }
}

impl Hash for MessageQop {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}

#[derive(Clone, Debug, Eq)]
pub struct NameAddress {
    display_name: Option<String>,
    uri: Uri,
}

impl NameAddress {
    pub(crate) fn new(uri: Uri, display_name: Option<String>) -> Self {
        Self { display_name, uri }
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }
}

impl std::fmt::Display for NameAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}<{}>",
            match &self.display_name {
                Some(display_name) => format!("\"{display_name}\" "),
                None => "".to_string(),
            },
            self.uri
        )
    }
}

impl PartialEq for NameAddress {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl PartialEq<&NameAddress> for NameAddress {
    fn eq(&self, other: &&NameAddress) -> bool {
        self == *other
    }
}

impl PartialEq<NameAddress> for &NameAddress {
    fn eq(&self, other: &NameAddress) -> bool {
        *self == other
    }
}

impl Hash for NameAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
    }
}

impl From<NameAddress> for Uri {
    fn from(value: NameAddress) -> Self {
        value.uri
    }
}
