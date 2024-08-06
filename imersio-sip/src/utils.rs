use std::collections::HashSet;
use std::hash::Hash;

pub(crate) fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash + std::fmt::Debug,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}

pub(crate) fn escape<F>(input: &str, f: F) -> String
where
    F: Fn(char) -> bool,
{
    input
        .chars()
        .map(|c| {
            if f(c) {
                format!("{}", c)
            } else {
                format!("%{0:x}", c as u32)
            }
        })
        .collect::<String>()
}

pub(crate) fn compare_vectors<I>(first: I, second: I) -> bool
where
    I: IntoIterator,
    I::Item: Hash + Eq + PartialEq,
{
    let first_values: HashSet<_> = first.into_iter().collect();
    let second_values: HashSet<_> = second.into_iter().collect();
    first_values == second_values
}
