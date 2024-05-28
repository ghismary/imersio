use std::hash::Hash;

use crate::utils::partial_eq_refs;

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

partial_eq_refs!(Algorithm);

impl Hash for Algorithm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().to_ascii_lowercase().hash(state);
    }
}
