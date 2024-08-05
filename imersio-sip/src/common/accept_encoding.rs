use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::utils::compare_vectors;
use crate::AcceptParameter;
use crate::ContentEncoding;

/// Representation of the list of encodings from an `AcceptEncodingHeader`.
///
/// This is usable as an iterator.
pub type AcceptEncodings = HeaderValueCollection<AcceptEncoding>;

impl AcceptEncodings {
    /// Tell whether the encodings contain the given encoding.
    pub fn contains(&self, encoding: &str) -> bool {
        self.iter().any(|e| e.encoding == encoding)
    }

    /// Get the `Encoding` corresponding to the given encoding name.
    pub fn get(&self, encoding: &str) -> Option<&AcceptEncoding> {
        self.iter().find(|e| e.encoding == encoding)
    }
}

/// Representation of an encoding from an `Accept-Encoding` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct AcceptEncoding {
    encoding: ContentEncoding,
    parameters: Vec<AcceptParameter>,
}

impl AcceptEncoding {
    pub(crate) fn new(encoding: ContentEncoding, parameters: Vec<AcceptParameter>) -> Self {
        AcceptEncoding {
            encoding,
            parameters,
        }
    }

    /// Get the encoding.
    pub fn encoding(&self) -> &str {
        self.encoding.as_ref()
    }

    /// Get a reference to the parameters for the encoding.
    pub fn parameters(&self) -> &Vec<AcceptParameter> {
        &self.parameters
    }

    /// Get the value of the `q` parameter for the encoding, if it has one.
    pub fn q(&self) -> Option<f32> {
        self.parameters
            .iter()
            .find(|param| matches!(param, AcceptParameter::Q(_)))
            .and_then(|param| param.q())
    }
}

impl std::fmt::Display for AcceptEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.encoding,
            if self.parameters.is_empty() { "" } else { ";" },
            join(&self.parameters, ";")
        )
    }
}

impl PartialEq for AcceptEncoding {
    fn eq(&self, other: &Self) -> bool {
        self.encoding == other.encoding && compare_vectors(self.parameters(), other.parameters())
    }
}

impl Hash for AcceptEncoding {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.encoding.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}
