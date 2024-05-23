//! TODO

use std::{cmp::Ordering, hash::Hash};

use crate::Uri;

/// Representation of a generic parameter.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenericParameter {
    key: String,
    value: Option<String>,
}

impl GenericParameter {
    /// Create a `GenericParam`.
    pub fn new(key: String, value: Option<String>) -> Self {
        Self { key, value }
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AcceptParameter {
    Q(String),
    Other(String, Option<String>),
}

impl AcceptParameter {
    pub(crate) fn new(key: String, value: Option<String>) -> Self {
        match (key.as_str(), value.as_deref()) {
            ("q", Some(value)) => Self::Q(value.to_string()),
            _ => Self::Other(key.to_string(), value.map(Into::into)),
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
            Self::Other(key, _) => key,
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Q(value) => Some(value),
            Self::Other(_, value) => value.as_deref(),
        }
    }
}

impl std::fmt::Display for AcceptParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Q(value) => write!(f, "q={value}"),
            Self::Other(key, value) => write!(
                f,
                "{}{}{}",
                key,
                if value.is_some() { "=" } else { "" },
                value.as_deref().unwrap_or_default()
            ),
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
        Self::Other(value.key().to_string(), value.value().map(Into::into))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Algorithm {
    Md5,
    Md5Sess,
    Other(String),
}

impl Algorithm {
    pub(crate) fn new(algo: String) -> Self {
        match algo.as_str() {
            "MD5" => Self::Md5,
            "MD5-Sess" => Self::Md5Sess,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum MessageQop {
    Auth,
    AuthInt,
    Other(String),
}

impl MessageQop {
    pub(crate) fn new(qop: String) -> Self {
        match qop.as_str() {
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

#[derive(Clone, Debug)]
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

impl Eq for NameAddress {}

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
