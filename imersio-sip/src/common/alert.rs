use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::utils::compare_vectors;
use crate::AbsoluteUri;
use crate::AcceptParameter;

/// Representation of the list of alerts from an `AlertInfoHeader`.
///
/// This is usable as an iterator.
pub type Alerts = HeaderValueCollection<Alert>;

impl Alerts {
    /// Tell whether `Alerts` contain the given `Uri`.
    pub fn contains(&self, uri: &AbsoluteUri) -> bool {
        self.iter().any(|a| a.uri == uri)
    }

    /// Get the `Alert` corresponding to the given `Uri`.
    pub fn get(&self, uri: &AbsoluteUri) -> Option<&Alert> {
        self.iter().find(|a| a.uri == uri)
    }
}

/// Representation of an alert contained in an `Alert-Info` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct Alert {
    uri: AbsoluteUri,
    parameters: Vec<AcceptParameter>,
}

impl Alert {
    pub(crate) fn new(uri: AbsoluteUri, parameters: Vec<AcceptParameter>) -> Self {
        Alert { uri, parameters }
    }

    /// Get a reference to the uri contained in the `Alert`.
    pub fn uri(&self) -> &AbsoluteUri {
        &self.uri
    }

    /// Get a reference to the parameters contained in the `Alert`.
    pub fn parameters(&self) -> &Vec<AcceptParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for Alert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}>{}{}",
            self.uri,
            if self.parameters.is_empty() { "" } else { ";" },
            join(&self.parameters, ";")
        )
    }
}

impl PartialEq for Alert {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri && compare_vectors(self.parameters(), other.parameters())
    }
}

impl Hash for Alert {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}
