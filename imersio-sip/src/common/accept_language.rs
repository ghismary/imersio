use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::utils::compare_vectors;
use crate::AcceptParameter;

/// Representation of the list of languages from an `AcceptLanguageHeader`.
///
/// This is usable as an iterator.
pub type AcceptLanguages = HeaderValueCollection<AcceptLanguage>;

impl AcceptLanguages {
    /// Tell whether `Languages` contains the given language.
    pub fn contains(&self, language: &str) -> bool {
        self.iter().any(|l| l.language == language)
    }

    /// Get the `Language` corresponding to the given language name.
    pub fn get(&self, language: &str) -> Option<&AcceptLanguage> {
        self.iter().find(|l| l.language == language)
    }
}

/// Representation of a language contained in an `Accept-Language` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct AcceptLanguage {
    language: String,
    parameters: Vec<AcceptParameter>,
}

impl AcceptLanguage {
    pub(crate) fn new<S: Into<String>>(language: S, parameters: Vec<AcceptParameter>) -> Self {
        AcceptLanguage {
            language: language.into(),
            parameters,
        }
    }

    /// Get the language.
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Get a reference to the parameters of the `Language`.
    pub fn parameters(&self) -> &Vec<AcceptParameter> {
        &self.parameters
    }

    /// Get the value of the `q` parameter for the language, if it has one.
    pub fn q(&self) -> Option<f32> {
        self.parameters
            .iter()
            .find(|param| matches!(param, AcceptParameter::Q(_)))
            .and_then(|param| param.q())
    }
}

impl std::fmt::Display for AcceptLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.language.to_ascii_lowercase(),
            if self.parameters.is_empty() { "" } else { ";" },
            join(&self.parameters, ";")
        )
    }
}

impl PartialEq for AcceptLanguage {
    fn eq(&self, other: &Self) -> bool {
        self.language.eq_ignore_ascii_case(&other.language)
            && compare_vectors(self.parameters(), other.parameters())
    }
}

impl Hash for AcceptLanguage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.language.to_ascii_lowercase().hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}
