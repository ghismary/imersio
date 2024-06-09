use derive_more::Deref;
use itertools::join;
use std::{hash::Hash, ops::Deref};

use crate::utils::compare_vectors;

#[derive(Clone, Debug, Deref, Eq)]
pub struct HeaderValueCollection<T>(Vec<T>)
where
    T: Eq + PartialEq + Hash;

impl<T> From<Vec<T>> for HeaderValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> std::fmt::Display for HeaderValueCollection<T>
where
    T: std::fmt::Display + Eq + PartialEq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", join(self.deref(), ", "))
    }
}

impl<T> PartialEq for HeaderValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        compare_vectors(self.deref(), other.deref())
    }
}

impl<T> PartialEq<&HeaderValueCollection<T>> for HeaderValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    fn eq(&self, other: &&HeaderValueCollection<T>) -> bool {
        self == *other
    }
}

impl<T> PartialEq<HeaderValueCollection<T>> for &HeaderValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    fn eq(&self, other: &HeaderValueCollection<T>) -> bool {
        *self == other
    }
}
