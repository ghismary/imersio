use itertools::{Itertools, join};
use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::Deref;

use crate::Uri;
use crate::utils::compare_vectors;

/// Representation of the list of uris in a `domain` parameter of a `Proxy-Authenticate` header.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Eq, derive_more::Deref, derive_more::From)]
pub struct DomainUris(Vec<DomainUri>);

impl std::fmt::Display for DomainUris {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#""{}""#, join(self.deref(), " "))
    }
}

impl PartialEq for DomainUris {
    fn eq(&self, other: &Self) -> bool {
        compare_vectors(self.deref(), other.deref())
    }
}

impl Hash for DomainUris {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.iter().sorted().for_each(|value| value.hash(state))
    }
}

/// Representation of a uri contained in a `domain` parameter of a `Proxy-Authenticate` header.
#[derive(Clone, Debug, Eq)]
pub enum DomainUri {
    /// A full uri for the domain.
    Uri(Uri),
    /// An absolute path for the domain.
    AbsPath(String),
}

impl std::fmt::Display for DomainUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uri(uri) => write!(f, "{}", uri),
            Self::AbsPath(path) => write!(f, "{}", path),
        }
    }
}

impl PartialEq for DomainUri {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Uri(a), Self::Uri(b)) => a == b,
            (Self::AbsPath(a), Self::AbsPath(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for DomainUri {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DomainUri {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl Hash for DomainUri {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Uri(uri) => uri.hash(state),
            Self::AbsPath(path) => path.hash(state),
        }
    }
}
