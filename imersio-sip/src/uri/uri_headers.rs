use std::hash::Hash;

use partial_eq_refs::PartialEqRefs;

use crate::{parser::is_unreserved, uri::parser::is_hnv_unreserved, utils::escape};

/// Representation of an URI header list.
#[derive(Clone, Debug, Default, Eq, PartialEqRefs)]
pub struct UriHeaders(Vec<(String, String)>);

impl UriHeaders {
    pub(crate) fn new<S: Into<String>>(headers: Vec<(S, S)>) -> Self {
        Self(
            headers
                .into_iter()
                .map(|(n, v)| (n.into(), v.into()))
                .collect(),
        )
    }

    /// Tell whether the headers list is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of headers.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Tell whether the headers list contains a header with the given name.
    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|(n, _)| n == name)
    }

    /// Get the header corresponding to the given name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_ref())
    }
}

impl std::fmt::Display for UriHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        escape(k, |c| { is_unreserved(c) || is_hnv_unreserved(c) }),
                        escape(v, |c| { is_unreserved(c) || is_hnv_unreserved(c) })
                    )
                })
                .collect::<Vec<String>>()
                .join("&"),
        )
    }
}

impl PartialEq for UriHeaders {
    fn eq(&self, other: &Self) -> bool {
        for (sk, sv) in &self.0 {
            if let Some(ov) = other.get(sk) {
                if sv != ov {
                    return false;
                }
            } else {
                return false;
            }
        }

        for (ok, ov) in &other.0 {
            if let Some(sv) = self.get(ok) {
                if ov != sv {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl Hash for UriHeaders {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut sorted_headers: Vec<(String, String)> = self
            .0
            .iter()
            .map(|(key, value)| (key.to_ascii_lowercase(), value.to_ascii_lowercase()))
            .collect();
        sorted_headers.sort_by(|(a, _), (b, _)| a.cmp(b));
        sorted_headers.hash(state)
    }
}
