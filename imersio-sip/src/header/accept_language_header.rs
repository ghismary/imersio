use std::{collections::HashSet, hash::Hash};

use super::accept_header::AcceptParameter;

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
        if self.language != other.language {
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
        self.language.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::Header;
    use std::str::FromStr;

    #[test]
    fn test_valid_accept_language_header() {
        let header = Header::from_str("Accept-Language: da");
        assert!(header.is_ok());
        if let Header::AcceptLanguage(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 1);
            assert!(header.contains("da"));
            assert!(!header.contains("en-gb"));
            assert!(!header.contains("en"));
        } else {
            panic!("Not an Accept-Language header");
        }

        let header = Header::from_str("Accept-Language: da, en");
        assert!(header.is_ok());
        if let Header::AcceptLanguage(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 2);
            assert!(header.contains("da"));
            assert!(!header.contains("en-gb"));
            assert!(header.contains("en"));
        } else {
            panic!("Not an Accept-Language header");
        }

        let header = Header::from_str("Accept-Language: da     ,  en  ,     en-gb");
        assert!(header.is_ok());
        if let Header::AcceptLanguage(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 3);
            assert!(header.contains("da"));
            assert!(header.contains("en-gb"));
            assert!(header.contains("en"));
        } else {
            panic!("Not an Accept-Language header");
        }

        let header = Header::from_str("Accept-Language:");
        assert!(header.is_ok());
        if let Header::AcceptLanguage(header) = header.unwrap() {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains("da"));
            assert!(!header.contains("en-gb"));
            assert!(!header.contains("en"));
        } else {
            panic!("Not an Accept-Language header");
        }

        let header = Header::from_str("Accept-Language:   ");
        assert!(header.is_ok());
        if let Header::AcceptLanguage(header) = header.unwrap() {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains("da"));
            assert!(!header.contains("en-gb"));
            assert!(!header.contains("en"));
        } else {
            panic!("Not an Accept-Language header");
        }

        let header = Header::from_str("Accept-Language: da, en-gb;q=0.8, en;q=0.7");
        assert!(header.is_ok());
        if let Header::AcceptLanguage(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 3);
            assert!(header.contains("da"));
            assert!(header.contains("en-gb"));
            assert!(header.contains("en"));
            // TODO: test parameters
        } else {
            panic!("Not an Accept-Language header");
        }
    }

    #[test]
    fn test_invalid_accept_language_header() {
        let header = Header::from_str("Accept-Language: 😁");
        assert!(header.is_err());
    }

    #[test]
    fn test_accept_language_header_equality() {
        let first_header = Header::from_str("Accept-Language: fr");
        let second_header = Header::from_str("Accept-Language: fr");
        if let (Header::AcceptLanguage(first_header), Header::AcceptLanguage(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Accept-Language header");
        }

        let first_header = Header::from_str("Accept-Language: fr, en");
        let second_header = Header::from_str("Accept-Language: en, fr");
        if let (Header::AcceptLanguage(first_header), Header::AcceptLanguage(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Accept-Language header");
        }
    }

    #[test]
    fn test_accept_language_header_inequality() {
        let first_header = Header::from_str("Accept-Language: fr");
        let second_header = Header::from_str("Accept-Language: en");
        if let (Header::AcceptLanguage(first_header), Header::AcceptLanguage(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept-Language header");
        }

        let first_header = Header::from_str("Accept-Language: fr, en");
        let second_header = Header::from_str("Accept-Language: en");
        if let (Header::AcceptLanguage(first_header), Header::AcceptLanguage(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept-Language header");
        }

        let first_header = Header::from_str("Accept-Language: en");
        let second_header = Header::from_str("Accept-Language: fr, en");
        if let (Header::AcceptLanguage(first_header), Header::AcceptLanguage(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Accept-Language header");
        }
    }
}
