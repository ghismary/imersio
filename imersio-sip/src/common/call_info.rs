use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::utils::compare_vectors;
use crate::AbsoluteUri;
use crate::CallInfoParameter;

/// Representation of the list of call information from a `Call-Info` header.
///
/// This is usable as an iterator.
pub type CallInfos = HeaderValueCollection<CallInfo>;

impl CallInfos {
    /// Tell whether Call-Info header contains the given `AbsoluteUri`.
    pub fn contains(&self, uri: &AbsoluteUri) -> bool {
        self.iter().any(|info| info.uri == uri)
    }

    /// Get the `CallInfo` corresponding to the given `AbsoluteUri`.
    pub fn get(&self, uri: &AbsoluteUri) -> Option<&CallInfo> {
        self.iter().find(|info| info.uri == uri)
    }
}

/// Representation of a call info, containing its uri and parameters.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct CallInfo {
    uri: AbsoluteUri,
    parameters: Vec<CallInfoParameter>,
}

impl CallInfo {
    pub(crate) fn new(uri: AbsoluteUri, parameters: Vec<CallInfoParameter>) -> Self {
        CallInfo { uri, parameters }
    }

    /// Get a reference to the uri of the `CallInfo`.
    pub fn uri(&self) -> &AbsoluteUri {
        &self.uri
    }

    /// Get a reference to the parameters of the `CallInfo`.
    pub fn parameters(&self) -> &Vec<CallInfoParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for CallInfo {
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

impl PartialEq for CallInfo {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri && compare_vectors(self.parameters(), other.parameters())
    }
}

impl Hash for CallInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}
