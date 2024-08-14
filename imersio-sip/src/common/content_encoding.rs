use derive_more::Display;
use nom::error::convert_error;
use partial_eq_refs::PartialEqRefs;
use std::cmp::Ordering;
use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::Error;

/// Representation of the list of encodings in a `Content-Encoding` header.
///
/// This is usable as an iterator.
pub type ContentEncodings = ValueCollection<ContentEncoding>;

/// Representation of an encoding in a `Content-Encoding` header.
#[derive(Clone, Debug, Display, Eq, PartialEqRefs)]
#[display("{}", self.0.to_ascii_lowercase())]
pub struct ContentEncoding(String);

impl ContentEncoding {
    pub(crate) fn new<S: Into<String>>(encoding: S) -> Self {
        Self(encoding.into())
    }
}

impl PartialEq for ContentEncoding {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<str> for ContentEncoding {
    fn eq(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<ContentEncoding> for str {
    fn eq(&self, other: &ContentEncoding) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<&str> for ContentEncoding {
    fn eq(&self, other: &&str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq<ContentEncoding> for &str {
    fn eq(&self, other: &ContentEncoding) -> bool {
        self.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialOrd for ContentEncoding {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContentEncoding {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .to_ascii_lowercase()
            .cmp(&other.0.to_ascii_lowercase())
    }
}

impl Hash for ContentEncoding {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state)
    }
}

impl AsRef<str> for ContentEncoding {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ContentEncoding {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::content_coding(value) {
            Ok((rest, encoding)) => {
                if !rest.is_empty() {
                    Err(Error::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(encoding)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(Error::InvalidContentEncoding(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidContentEncoding(format!(
                "Incomplete content encoding `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use crate::parser::{token, ParserResult};
    use crate::ContentEncoding;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, error::context};

    #[inline]
    pub(crate) fn content_coding(input: &str) -> ParserResult<&str, ContentEncoding> {
        context("content_coding", map(token, ContentEncoding::new))(input)
    }

    pub(crate) fn codings(input: &str) -> ParserResult<&str, ContentEncoding> {
        context(
            "codings",
            alt((content_coding, map(tag("*"), ContentEncoding::new))),
        )(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_content_encoding_eq() {
        assert_eq!(ContentEncoding::try_from("*").unwrap(), "*");
        assert_eq!(ContentEncoding::try_from("gzip").unwrap(), "gzip");
    }

    #[test]
    fn test_content_encoding_eq_different_case() {
        assert_eq!(ContentEncoding::try_from("tar").unwrap(), "TAR");
    }

    #[test]
    fn test_valid_content_encoding() {
        assert_ok!(ContentEncoding::try_from("gzip"));
    }

    #[test]
    fn test_valid_content_encoding_wildcard() {
        assert_ok!(ContentEncoding::try_from("*"));
    }

    #[test]
    fn test_invalid_content_encoding_empty() {
        assert_err!(ContentEncoding::try_from(""));
    }

    #[test]
    fn test_invalid_content_encoding_with_invalid_character() {
        assert_err!(ContentEncoding::try_from("en-😁"));
    }

    #[test]
    fn test_valid_content_encoding_with_remaining_data() {
        assert!(ContentEncoding::try_from("gzip anything")
            .is_err_and(|e| e == Error::RemainingUnparsedData(" anything".to_string())));
    }
}
