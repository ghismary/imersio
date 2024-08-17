use itertools::join;
use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::utils::compare_vectors;
use crate::AcceptParameter;

/// Representation of the list of languages from an `AcceptLanguageHeader`.
///
/// This is usable as an iterator.
pub type AcceptLanguages = ValueCollection<AcceptLanguage>;

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
#[derive(Clone, Debug, Eq)]
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

pub(crate) mod parser {
    use crate::common::accept_parameter::parser::accept_param;
    use crate::parser::{alpha, semi, ParserResult};
    use crate::AcceptLanguage;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, opt, recognize},
        error::context,
        multi::{many0, many_m_n},
        sequence::{pair, preceded},
    };

    fn language_range(input: &str) -> ParserResult<&str, &str> {
        context(
            "language_range",
            alt((
                recognize(pair(
                    many_m_n(1, 8, alpha),
                    opt(many0(pair(tag("-"), many_m_n(1, 8, alpha)))),
                )),
                tag("*"),
            )),
        )(input)
    }

    pub(crate) fn language(input: &str) -> ParserResult<&str, AcceptLanguage> {
        context(
            "language",
            map(
                pair(language_range, many0(preceded(semi, accept_param))),
                |(language, params)| AcceptLanguage::new(language, params),
            ),
        )(input)
    }
}
