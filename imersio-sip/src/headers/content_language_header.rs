//! SIP Content-Language header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{ContentLanguage, ContentLanguages};

/// Representation of a Content-Language header.
///
/// [[RFC3261, Section 20.13](https://datatracker.ietf.org/doc/html/rfc3261#section-20.13)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct ContentLanguageHeader {
    #[partial_eq_ignore]
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
    crate::headers::generic_header_accessors!(header);

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

pub(crate) mod parser {
    use crate::common::content_language::parser::language_tag;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{ContentLanguageHeader, Header};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn content_language(input: &str) -> ParserResult<&str, Header> {
        context(
            "Content-Language header",
            map(
                tuple((
                    tag_no_case("Content-Language"),
                    hcolon,
                    cut(consumed(separated_list1(comma, language_tag))),
                )),
                |(name, separator, (value, languages))| {
                    Header::ContentLanguage(ContentLanguageHeader::new(
                        GenericHeader::new(name, separator, value),
                        languages,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        ContentLanguageHeader, Header,
    };
    use claims::assert_ok;

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
        let header = Header::try_from("content-LanguAge:  fr , EN-GB");
        if let Header::ContentLanguage(header) = header.unwrap() {
            assert_eq!(header.to_string(), "content-LanguAge:  fr , EN-GB");
            assert_eq!(header.to_normalized_string(), "Content-Language: fr, en-gb");
            assert_eq!(header.to_compact_string(), "Content-Language: fr, en-gb");
        }
    }
}
