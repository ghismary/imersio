use std::{collections::HashSet, hash::Hash, ops::Deref};

#[derive(Clone, Debug, Eq)]
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
        write!(
            f,
            "{}",
            self.iter()
                .map(|value| value.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl<T> PartialEq for HeaderValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        let self_values: HashSet<_> = self.iter().collect();
        let other_values: HashSet<_> = other.iter().collect();
        self_values == other_values
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

impl<T> IntoIterator for HeaderValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Deref for HeaderValueCollection<T>
where
    T: Eq + PartialEq + Hash,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}
