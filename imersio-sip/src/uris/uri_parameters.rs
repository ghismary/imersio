//! TODO

use std::{hash::Hash, ops::Deref};

use partial_eq_refs::PartialEqRefs;

use crate::uris::parser::is_param_unreserved;
use crate::{parser::is_unreserved, utils::escape};

/// Representation of an URI parameter list.
#[derive(Clone, Debug, Default, Eq, PartialEqRefs)]
pub struct UriParameters(Vec<(String, Option<String>)>);

impl UriParameters {
    pub(crate) fn new<S: Into<String>>(parameters: Vec<(S, Option<S>)>) -> Self {
        Self(
            parameters
                .into_iter()
                .map(|(key, value)| (key.into(), value.map(Into::into)))
                .collect::<Vec<(String, Option<String>)>>(),
        )
    }

    /// Tell whether the parameters list is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of parameters.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Tell whether the parameters list contains a parameter with the given
    /// name.
    pub fn contains(&self, name: &str) -> bool {
        self.iter().any(|(n, _)| n == name)
    }

    /// Get the parameter corresponding to the given name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .and_then(|(_, v)| v.as_deref())
    }
}

impl std::fmt::Display for UriParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}{}{}",
                        escape(k, |b| { is_unreserved(b) || is_param_unreserved(b) }),
                        if v.is_some() { "=" } else { "" },
                        escape(v.as_deref().unwrap_or_default(), |b| {
                            is_unreserved(b) || is_param_unreserved(b)
                        })
                    )
                })
                .collect::<Vec<String>>()
                .join(";"),
        )
    }
}

impl Deref for UriParameters {
    type Target = Vec<(String, Option<String>)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for UriParameters {
    fn eq(&self, other: &Self) -> bool {
        for (sk, sv) in &self.0 {
            for (ok, ov) in &other.0 {
                if sk.eq_ignore_ascii_case(ok)
                    && sv.as_ref().map(|s| s.to_ascii_lowercase())
                        != ov.as_ref().map(|s| s.to_ascii_lowercase())
                {
                    return false;
                }
            }
        }

        let stransport = self.get("transport");
        let otransport = other.get("transport");
        match (stransport, otransport) {
            (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}

impl Hash for UriParameters {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut sorted_params: Vec<(String, Option<String>)> = self
            .0
            .iter()
            .map(|(key, value)| {
                (
                    key.to_ascii_lowercase(),
                    value.as_ref().map(|value| value.to_ascii_lowercase()),
                )
            })
            .collect();
        sorted_params.sort_by(|(a, _), (b, _)| a.cmp(b));
        sorted_params.hash(state)
    }
}
