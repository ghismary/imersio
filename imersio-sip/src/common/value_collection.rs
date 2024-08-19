use derive_more::{Deref, DerefMut};
use itertools::join;
use std::hash::Hash;

use crate::utils::compare_vectors;

#[derive(Clone, Debug, Deref, DerefMut, Eq)]
pub struct ValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    #[deref]
    #[deref_mut]
    values: Vec<T>,
    separator: &'static str,
}

impl<T> ValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    pub fn set_separator(self, separator: &'static str) -> Self {
        Self {
            values: self.values,
            separator,
        }
    }
}

impl<T> From<Vec<T>> for ValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    fn from(value: Vec<T>) -> Self {
        Self {
            values: value,
            separator: ", ",
        }
    }
}

impl<T> std::fmt::Display for ValueCollection<T>
where
    T: std::fmt::Display + Eq + PartialEq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", join(self.deref(), self.separator))
    }
}

impl<T> PartialEq for ValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        compare_vectors(self.deref(), other.deref())
    }
}

impl<T> Default for ValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    fn default() -> Self {
        Self {
            values: vec![],
            separator: ", ",
        }
    }
}
