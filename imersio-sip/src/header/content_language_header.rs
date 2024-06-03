use std::{hash::Hash, str::FromStr};

use partial_eq_refs::PartialEqRefs;

use crate::{common::header_value_collection::HeaderValueCollection, Error, HeaderAccessor};

use super::generic_header::GenericHeader;

/// Representation of a Content-Language header.
///
/// [[RFC3261, Section 20.13](https://datatracker.ietf.org/doc/html/rfc3261#section-20.13)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ContentLanguageHeader {
    header: GenericHeader,
    languages: ContentLanguages,
}

impl ContentLanguageHeader {
    pub(crate) fn new(header: GenericHeader, languages: Vec<ContentLanguage>) -> Self {
        Self {
            header,
            languages: languages.into(),
        }
    }

    /// Get a reference to the languages from the Content-Language header.
    pub fn languages(&self) -> &ContentLanguages {
        &self.languages
    }
}

impl HeaderAccessor for ContentLanguageHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Content-Language")
    }
    fn normalized_value(&self) -> String {
        self.languages.to_string()
    }
}

impl std::fmt::Display for ContentLanguageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq<ContentLanguageHeader> for ContentLanguageHeader {
    fn eq(&self, other: &ContentLanguageHeader) -> bool {
        self.languages == other.languages
    }
}

/// Representation of the list of languages in a `Content-Language` header.
///
/// This is usable as an iterator.
pub type ContentLanguages = HeaderValueCollection<ContentLanguage>;

/// Representation of a language contained in an `Content-Language` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ContentLanguage(String);

impl ContentLanguage {
    pub(crate) fn new<S: Into<String>>(language: S) -> Self {
        Self(language.into())
    }
}

impl std::fmt::Display for ContentLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_ascii_lowercase())
    }
}

impl PartialEq for ContentLanguage {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<str> for ContentLanguage {
    fn eq(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<ContentLanguage> for str {
    fn eq(&self, other: &ContentLanguage) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<&str> for ContentLanguage {
    fn eq(&self, other: &&str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<ContentLanguage> for &str {
    fn eq(&self, other: &ContentLanguage) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl AsRef<str> for ContentLanguage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for ContentLanguage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::header::parser::language_tag(s.as_bytes())
            .map(|(_, language)| language)
            .map_err(|_| Error::InvalidContentLanguage(s.to_string()))
    }
}

impl Hash for ContentLanguage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::ContentLanguageHeader;
    use crate::{
        header::tests::{header_equality, header_inequality, invalid_header, valid_header},
        Header, HeaderAccessor,
    };
    use claims::assert_ok;
    use std::str::FromStr;

    valid_header!(ContentLanguage, ContentLanguageHeader, "Content-Language");
    header_equality!(ContentLanguage, "Content-Language");
    header_inequality!(ContentLanguage, "Content-Language");

    #[test]
    fn test_valid_content_language_header() {
        valid_header("Content-Language: fr", |header| {
            assert_eq!(header.languages().len(), 1);
            assert_eq!(header.languages().first().unwrap(), "fr");
        });
    }

    #[test]
    fn test_valid_content_language_header_with_several_languages() {
        valid_header("Content-Language: fr, en-GB", |header| {
            assert_eq!(header.languages().len(), 2);
            assert_eq!(
                header
                    .languages()
                    .iter()
                    .map(|l| l.as_ref())
                    .collect::<Vec<&str>>(),
                vec!["fr", "en-GB"]
            );
        });
    }

    #[test]
    fn test_invalid_content_language_header_empty() {
        invalid_header("Content-Language:");
    }

    #[test]
    fn test_invalid_content_language_header_empty_with_space_characters() {
        invalid_header("Content-Language:    ");
    }

    #[test]
    fn test_invalid_content_language_header_with_invalid_character() {
        invalid_header("Content-Language: 😁");
    }

    #[test]
    fn test_content_language_header_equality_same_header_with_space_characters_differences() {
        header_equality("Content-Language: fr", "Content-Language:  fr");
    }

    #[test]
    fn test_content_language_header_equality_same_languages_in_a_different_order() {
        header_equality("Content-Language: fr, en", "Content-Language: en, fr");
    }

    #[test]
    fn test_content_language_header_equality_same_languages_with_different_cases() {
        header_equality("Content-Language: fr", "content-language: FR");
    }

    #[test]
    fn test_content_language_header_inequality_with_different_languages() {
        header_inequality("Content-Language: fr", "Content-Language: en");
    }

    #[test]
    fn test_content_language_header_inequality_with_first_having_more_languages_than_the_second() {
        header_inequality("Content-Language: fr, en", "Content-Language: en");
    }

    #[test]
    fn test_content_language_header_inequality_with_first_having_less_languages_than_the_second() {
        header_inequality("Content-Language: fr", "Content-Language: en, fr");
    }

    #[test]
    fn test_content_language_header_to_string() {
        let header = Header::from_str("content-LanguAge:  fr , EN-GB");
        if let Header::ContentLanguage(header) = header.unwrap() {
            assert_eq!(header.to_string(), "content-LanguAge:  fr , EN-GB");
            assert_eq!(header.to_normalized_string(), "Content-Language: fr, en-gb");
            assert_eq!(header.to_compact_string(), "Content-Language: fr, en-gb");
        }
    }
}
