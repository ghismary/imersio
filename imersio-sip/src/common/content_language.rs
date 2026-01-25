use nom_language::error::convert_error;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::SipError;

/// Representation of the list of languages in a `Content-Language` header.
///
/// This is usable as an iterator.
pub type ContentLanguages = ValueCollection<ContentLanguage>;

/// Representation of a language contained in an `Content-Language` header.
#[derive(Clone, Debug, Eq)]
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

impl PartialOrd for ContentLanguage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContentLanguage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .to_ascii_lowercase()
            .cmp(&other.0.to_ascii_lowercase())
    }
}

impl Hash for ContentLanguage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state)
    }
}

impl AsRef<str> for ContentLanguage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ContentLanguage {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::language_tag(value) {
            Ok((rest, language)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(language)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidContentLanguage(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidContentLanguage(format!(
                "Incomplete content language `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use nom::{
        bytes::complete::tag,
        combinator::{map, recognize},
        multi::{many0, many_m_n},
        sequence::{pair, preceded},
        Parser,
    };

    use crate::{
        parser::{alpha, ParserResult},
        ContentLanguage,
    };

    #[inline]
    fn primary_tag(input: &str) -> ParserResult<&str, &str> {
        recognize(many_m_n(1, 8, alpha)).parse(input)
    }

    #[inline]
    fn subtag(input: &str) -> ParserResult<&str, &str> {
        primary_tag(input)
    }

    pub(crate) fn language_tag(input: &str) -> ParserResult<&str, ContentLanguage> {
        map(
            recognize(pair(primary_tag, many0(preceded(tag("-"), subtag)))),
            ContentLanguage::new,
        )
        .parse(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_content_language_eq() {
        assert_eq!(ContentLanguage::try_from("en-US").unwrap(), "en-US");
    }

    #[test]
    fn test_content_language_eq_different_case() {
        assert_eq!(ContentLanguage::try_from("fr-FR").unwrap(), "fr-fr");
    }

    #[test]
    fn test_valid_content_language_with_only_primary_tag() {
        assert_ok!(ContentLanguage::try_from("en"));
    }

    #[test]
    fn test_valid_content_language_with_subtag() {
        assert_ok!(ContentLanguage::try_from("en-GB"));
    }

    #[test]
    fn test_invalid_content_language_empty() {
        assert_err!(ContentLanguage::try_from(""));
    }

    #[test]
    fn test_invalid_content_language_with_invalid_character() {
        assert_err!(ContentLanguage::try_from("en-😁"));
    }

    #[test]
    fn test_valid_content_language_with_remaining_data() {
        assert!(ContentLanguage::try_from("en-US anything")
            .is_err_and(|e| e == SipError::RemainingUnparsedData(" anything".to_string())));
    }
}
