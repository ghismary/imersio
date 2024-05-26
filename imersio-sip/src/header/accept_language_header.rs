use std::{collections::HashSet, hash::Hash};

use crate::common::AcceptParameter;

#[derive(Clone, Debug)]
pub struct AcceptLanguageHeader(Vec<Language>);

impl AcceptLanguageHeader {
    pub(crate) fn new(languages: Vec<Language>) -> Self {
        AcceptLanguageHeader(languages)
    }

    /// Tells whether the Accept-Language header is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of languages in the Accept-Language header.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Tells whether Accept-Language header contains the given languages.
    pub fn contains(&self, language: &str) -> bool {
        self.0.iter().any(|l| l.language == language)
    }

    /// Gets the `Language` corresponding to the given language name.
    pub fn get(&self, language: &str) -> Option<&Language> {
        self.0.iter().find(|l| l.language == language)
    }
}

impl std::fmt::Display for AcceptLanguageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Accept-Language: {}",
            self.0
                .iter()
                .map(|language| language.to_string())
                .collect::<Vec<String>>()
                .join(" ,")
        )
    }
}

impl PartialEq for AcceptLanguageHeader {
    fn eq(&self, other: &Self) -> bool {
        let self_languages: HashSet<_> = self.0.iter().collect();
        let other_languages: HashSet<_> = other.0.iter().collect();
        self_languages == other_languages
    }
}

impl PartialEq<&AcceptLanguageHeader> for AcceptLanguageHeader {
    fn eq(&self, other: &&AcceptLanguageHeader) -> bool {
        self == *other
    }
}

impl PartialEq<AcceptLanguageHeader> for &AcceptLanguageHeader {
    fn eq(&self, other: &AcceptLanguageHeader) -> bool {
        *self == other
    }
}

impl Eq for AcceptLanguageHeader {}

#[derive(Clone, Debug)]
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

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn parameters(&self) -> &Vec<AcceptParameter> {
        &self.parameters
    }

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
            self.language,
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

impl PartialEq<&Language> for Language {
    fn eq(&self, other: &&Language) -> bool {
        self == *other
    }
}

impl PartialEq<Language> for &Language {
    fn eq(&self, other: &Language) -> bool {
        *self == other
    }
}

impl Eq for Language {}

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
    use crate::Header;
    use std::str::FromStr;

    fn valid_header<F: FnOnce(AcceptLanguageHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert!(header.is_ok());
        if let Header::AcceptLanguage(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not an Accept-Language header");
        }
    }

    #[test]
    fn test_valid_accept_language_header_with_single_language() {
        valid_header("Accept-Language: da", |header| {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 1);
            assert!(header.contains("da"));
            assert!(!header.contains("en-gb"));
            assert!(!header.contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_with_several_languages() {
        valid_header("Accept-Language: da, en", |header| {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 2);
            assert!(header.contains("da"));
            assert!(!header.contains("en-gb"));
            assert!(header.contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_with_several_languages_and_space_characters() {
        valid_header("Accept-Language: da     ,  en  ,     en-gb", |header| {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 3);
            assert!(header.contains("da"));
            assert!(header.contains("en-gb"));
            assert!(header.contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_empty() {
        valid_header("Accept-Language:", |header| {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains("da"));
            assert!(!header.contains("en-gb"));
            assert!(!header.contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_empty_with_space_characters() {
        valid_header("Accept-Language:   ", |header| {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains("da"));
            assert!(!header.contains("en-gb"));
            assert!(!header.contains("en"));
        });
    }

    #[test]
    fn test_valid_accept_language_header_with_q_parameters() {
        valid_header("Accept-Language: da, en-gb;q=0.8, en;q=0.7", |header| {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 3);
            assert!(header.contains("da"));
            assert!(header.contains("en-gb"));
            assert!(header.contains("en"));
            let da_language = header.get("da").unwrap();
            assert!(da_language.parameters().is_empty());
            assert_eq!(da_language.q(), None);
            let en_gb_language = header.get("en-gb").unwrap();
            assert_eq!(en_gb_language.parameters().len(), 1);
            assert_eq!(en_gb_language.parameters().first().unwrap().key(), "q");
            assert_eq!(
                en_gb_language.parameters().first().unwrap().value(),
                Some("0.8")
            );
            let en_gb_language_q = en_gb_language.q();
            assert!(en_gb_language_q.is_some());
            assert!((en_gb_language_q.unwrap() - 0.8).abs() < 0.01);
            let en_language = header.get("en").unwrap();
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
        assert!(header.is_err());
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
}
