use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;
use std::ops::Deref;

use crate::common::value_collection::ValueCollection;
use crate::utils::compare_vectors;
use crate::{GenericParameter, Uri};

/// Representation of the list of error uris from an `ErrorInfoHeader`.
///
/// This is usable as an iterator.
pub type ErrorUris = ValueCollection<ErrorUri>;

impl ErrorUris {
    /// Tell whether `ErrorUris` contain the given `Uri`.
    pub fn contains(&self, uri: &Uri) -> bool {
        self.iter().any(|a| a.uri == uri)
    }

    /// Get the `ErrorUri` corresponding to the given `Uri`.
    pub fn get(&self, uri: &Uri) -> Option<&ErrorUri> {
        self.iter().find(|a| a.uri == uri)
    }
}

/// Representation of an error uri contained in an `Error-Info` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ErrorUri {
    uri: Uri,
    parameters: Vec<GenericParameter>,
}

impl ErrorUri {
    pub(crate) fn new(uri: Uri, parameters: Vec<GenericParameter>) -> Self {
        ErrorUri { uri, parameters }
    }

    /// Get a reference to the uri contained in the `ErrorUri`.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Get a reference to the parameters contained in the `ErrorUri`.
    pub fn parameters(&self) -> &Vec<GenericParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for ErrorUri {
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

impl PartialEq for ErrorUri {
    fn eq(&self, other: &Self) -> bool {
        if self.uri != other.uri {
            return false;
        }

        compare_vectors(self.parameters.deref(), other.parameters().deref())
    }
}

impl Hash for ErrorUri {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

pub(crate) mod parser {
    use crate::common::generic_parameter::parser::generic_param;
    use crate::parser::{laquot, raquot, semi, ParserResult};
    use crate::uris::uri::parser::request_uri;
    use crate::ErrorUri;
    use nom::{
        combinator::map,
        multi::many0,
        sequence::{preceded, tuple},
    };

    pub(crate) fn error_uri(input: &str) -> ParserResult<&str, ErrorUri> {
        map(
            tuple((
                laquot,
                request_uri,
                raquot,
                many0(preceded(semi, generic_param)),
            )),
            |(_, uri, _, parameters)| ErrorUri::new(uri, parameters),
        )(input)
    }
}
