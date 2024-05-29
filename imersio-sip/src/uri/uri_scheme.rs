use std::hash::Hash;

/// Representation of the scheme of an URI.
#[derive(Clone, Debug, Eq)]
pub enum UriScheme {
    /// SIP protocol scheme.
    Sip,
    /// SIPS protocol scheme.
    Sips,
    /// Any other protocol scheme.
    Other(String),
}

impl UriScheme {
    /// SIP protocol scheme.
    pub const SIP: UriScheme = UriScheme::Sip;

    /// SIPS protocol scheme.
    pub const SIPS: UriScheme = UriScheme::Sips;

    /// Get a str representation of the scheme.
    pub fn as_str(&self) -> &str {
        match self {
            UriScheme::Sip => "sip",
            UriScheme::Sips => "sips",
            UriScheme::Other(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for UriScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for UriScheme {
    fn default() -> Self {
        UriScheme::SIP
    }
}

impl AsRef<str> for UriScheme {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for UriScheme {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (&UriScheme::Sip, &UriScheme::Sip) => true,
            (&UriScheme::Sips, &UriScheme::Sips) => true,
            (UriScheme::Other(a), UriScheme::Other(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl PartialEq<str> for UriScheme {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<UriScheme> for str {
    fn eq(&self, other: &UriScheme) -> bool {
        other == self
    }
}

impl PartialEq<&UriScheme> for UriScheme {
    fn eq(&self, other: &&UriScheme) -> bool {
        self == *other
    }
}

impl PartialEq<UriScheme> for &UriScheme {
    fn eq(&self, other: &UriScheme) -> bool {
        *self == other
    }
}

impl Hash for UriScheme {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            UriScheme::Sip => {
                state.write_u8(1);
            }
            UriScheme::Sips => {
                state.write_u8(2);
            }
            UriScheme::Other(value) => {
                state.write_u8(3);
                value.to_ascii_lowercase().hash(state);
            }
        }
    }
}
