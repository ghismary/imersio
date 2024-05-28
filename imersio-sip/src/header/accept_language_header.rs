use std::{collections::HashSet, hash::Hash};

use partial_eq_refs::PartialEqRefs;

use crate::{
    common::{accept_parameter::AcceptParameter, header_value_collection::HeaderValueCollection},
    HeaderAccessor,
};

use super::generic_header::GenericHeader;

/// Representation of an Accept-Language header.
///
/// The Accept-Language header field is used in requests to indicate the
/// preferred languages for reason phrases, session descriptions, or status
/// responses carried as message bodies in the response. If no
/// Accept-Language header field is present, the server SHOULD assume all
/// languages are acceptable to the client.
///
/// [[RFC3261, Section 20.3](https://datatracker.ietf.org/doc/html/rfc3261#section-20.3)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct AcceptLanguageHeader {
    header: GenericHeader,
    languages: Languages,
}

impl AcceptLanguageHeader {
    pub(crate) fn new(header: GenericHeader, languages: Vec<Language>) -> Self {
        AcceptLanguageHeader {
            header,
            languages: languages.into(),
        }
    }

    /// Get the `Languages` from the `Accept-Language` header.
    pub fn languages(&self) -> &Languages {
        &self.languages
    }
}

impl HeaderAccessor for AcceptLanguageHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Accept-Language")
    }
    fn normalized_value(&self) -> String {
        self.languages.to_string()
    }
}

impl std::fmt::Display for AcceptLanguageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for AcceptLanguageHeader {
    fn eq(&self, other: &Self) -> bool {
        self.languages == other.languages
    }
}

/// Representation of the list of languages from an `AcceptLanguageHeader`.
///
/// This is usable as an iterator.
pub type Languages = HeaderValueCollection<Language>;

impl Languages {
    /// Tell whether `Languages` contains the given language.
    pub fn contains(&self, language: &str) -> bool {
        self.iter().any(|l| l.language == language)
    }

    /// Get the `Language` corresponding to the given language name.
    pub fn get(&self, language: &str) -> Option<&Language> {
        self.iter().find(|l| l.language == language)
    }
}

/// Representation of a language contained in an `Accept-Language` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct Language {
    language: String,
    parameters: Vec<AcceptParameter>,
}

impl Language {
    pub(crate) fn new(language: String, parameters: Vec<AcceptParameter>) -> Self {
        Language {
            language,
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

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.language.to_ascii_lowercase(),
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl PartialEq for Language {
    fn eq(&self, other: &Self) -> bool {
        if !self.language.eq_ignore_ascii_case(&other.language) {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

impl Hash for Language {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.language.to_ascii_lowercase().hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::AcceptLanguageHeader;
    use crate::{Header, HeaderAccessor};
    use claim::{assert_err, assert_ok};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(AcceptLanguageHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert_ok!(&header);
        if let Header::AcceptLanguage(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not an Accept-Language header");
        }
    }

    #[test]
    fn test_valid_accept_language_header_with_single_language() {
        valid_header("Accept-Language: da", |header| {
            assert!(!header.languages().is_empty());
            assert_eq!(header.languages().len(), 1);
            assert!(header.languages().contains("da"));
            assert!(!header.languages().contains("en-gb"));
            assert!(!header.languages().contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_with_several_languages() {
        valid_header("Accept-Language: da, en", |header| {
            assert!(!header.languages().is_empty());
            assert_eq!(header.languages().len(), 2);
            assert!(header.languages().contains("da"));
            assert!(!header.languages().contains("en-gb"));
            assert!(header.languages().contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_with_several_languages_and_space_characters() {
        valid_header("Accept-Language: da     ,  en  ,     en-gb", |header| {
            assert!(!header.languages().is_empty());
            assert_eq!(header.languages().len(), 3);
            assert!(header.languages().contains("da"));
            assert!(header.languages().contains("en-gb"));
            assert!(header.languages().contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_empty() {
        valid_header("Accept-Language:", |header| {
            assert!(header.languages().is_empty());
            assert_eq!(header.languages().len(), 0);
            assert!(!header.languages().contains("da"));
            assert!(!header.languages().contains("en-gb"));
            assert!(!header.languages().contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_empty_with_space_characters() {
        valid_header("Accept-Language:   ", |header| {
            assert!(header.languages().is_empty());
            assert_eq!(header.languages().len(), 0);
            assert!(!header.languages().contains("da"));
            assert!(!header.languages().contains("en-gb"));
            assert!(!header.languages().contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_with_q_parameters() {
        valid_header("Accept-Language: da, en-gb;q=0.8, en;q=0.7", |header| {
            assert!(!header.languages().is_empty());
            assert_eq!(header.languages().len(), 3);
            assert!(header.languages().contains("da"));
            assert!(header.languages().contains("en-gb"));
            assert!(header.languages().contains("en"));
            let da_language = header.languages().get("da").unwrap();
            assert!(da_language.parameters().is_empty());
            assert_eq!(da_language.q(), None);
            let en_gb_language = header.languages().get("en-gb").unwrap();
            assert_eq!(en_gb_language.parameters().len(), 1);
            assert_eq!(en_gb_language.parameters().first().unwrap().key(), "q");
            assert_eq!(
                en_gb_language.parameters().first().unwrap().value(),
                Some("0.8")
            );
            let en_gb_language_q = en_gb_language.q();
            assert!(en_gb_language_q.is_some());
            assert!((en_gb_language_q.unwrap() - 0.8).abs() < 0.01);
            let en_language = header.languages().get("en").unwrap();
            assert_eq!(en_language.parameters().len(), 1);
            assert_eq!(en_language.parameters().first().unwrap().key(), "q");
            assert_eq!(
                en_language.parameters().first().unwrap().value(),
                Some("0.7")
            );
            let en_language_q = en_language.q();
            assert!(en_language_q.is_some());
            assert!((en_language_q.unwrap() - 0.7).abs() < 0.01);
        });
    }

    #[test]
    fn test_invalid_accept_language_header_with_invalid_characters() {
        let header = Header::from_str("Accept-Language: 😁");
        assert_err!(header);
    }

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::AcceptLanguage(first_header), Header::AcceptLanguage(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Accept-Language header");
        }
    }

    #[test]
    fn test_accept_language_header_equality_same_headers_with_space_characters_differences() {
        header_equality("Accept-Language: fr", "Accept-Language:  fr");
    }

    #[test]
    fn test_accept_language_header_equality_same_headers_with_languages_in_a_different_order() {
        header_equality("Accept-Language: fr, en", "Accept-Language: en, fr");
    }

    #[test]
    fn test_accept_language_header_equality_same_languages_with_different_cases() {
        header_equality("Accept-Language: fr, en", "accept-language: EN, FR");
    }

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::AcceptLanguage(first_header), Header::AcceptLanguage(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept-Language header");
        }
    }

    #[test]
    fn test_accept_language_header_inequality_with_different_languages() {
        header_inequality("Accept-Language: fr", "Accept-Language: en");
    }

    #[test]
    fn test_accept_language_header_inequality_with_first_header_having_more_languages_than_the_second(
    ) {
        header_inequality("Accept-Language: fr, en", "Accept-Language: en");
    }

    #[test]
    fn test_accept_language_header_inequality_with_first_header_having_less_languages_than_the_second(
    ) {
        header_inequality("Accept-Language: en", "Accept-Language: fr, en");
    }

    #[test]
    fn test_accept_language_header_to_string() {
        let header = Header::from_str("accept-language:  EN   , FR");
        if let Header::AcceptLanguage(header) = header.unwrap() {
            assert_eq!(header.to_string(), "accept-language:  EN   , FR");
            assert_eq!(header.to_normalized_string(), "Accept-Language: en, fr");
            assert_eq!(header.to_compact_string(), "Accept-Language: en, fr");
        }
    }
}
