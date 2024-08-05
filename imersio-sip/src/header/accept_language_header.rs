//! SIP Accept-Language header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::header::GenericHeader;
use crate::HeaderAccessor;
use crate::{AcceptLanguage, AcceptLanguages};

/// Representation of an Accept-Language header.
///
/// The Accept-Language header field is used in requests to indicate the
/// preferred languages for reason phrases, session descriptions, or status
/// responses carried as message bodies in the response. If no
/// Accept-Language header field is present, the server SHOULD assume all
/// languages are acceptable to the client.
///
/// [[RFC3261, Section 20.3](https://datatracker.ietf.org/doc/html/rfc3261#section-20.3)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct AcceptLanguageHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    languages: AcceptLanguages,
}

impl AcceptLanguageHeader {
    pub(crate) fn new(header: GenericHeader, languages: Vec<AcceptLanguage>) -> Self {
        Self {
            header,
            languages: languages.into(),
        }
    }

    /// Get the `Languages` from the `Accept-Language` header.
    pub fn languages(&self) -> &AcceptLanguages {
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

#[cfg(test)]
mod tests {
    use super::AcceptLanguageHeader;
    use crate::header::tests::{header_equality, header_inequality, invalid_header, valid_header};
    use crate::{Header, HeaderAccessor};
    use claims::assert_ok;

    valid_header!(AcceptLanguage, AcceptLanguageHeader, "Accept-Language");
    header_equality!(AcceptLanguage, "Accept-Language");
    header_inequality!(AcceptLanguage, "Accept-Language");

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
        invalid_header("Accept-Language: 😁");
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
        let header = Header::try_from("accept-language:  EN   , FR");
        if let Header::AcceptLanguage(header) = header.unwrap() {
            assert_eq!(header.to_string(), "accept-language:  EN   , FR");
            assert_eq!(header.to_normalized_string(), "Accept-Language: en, fr");
            assert_eq!(header.to_compact_string(), "Accept-Language: en, fr");
        }
    }
}
