use itertools::join;
use std::hash::Hash;
use std::ops::Deref;

use crate::utils::compare_vectors;
use crate::MediaParameter;
use crate::MediaRange;

/// Representation of a media type contained in a `ContentTypeHeader`.
#[derive(Clone, Debug, Eq)]
pub struct MediaType {
    media_range: MediaRange,
    parameters: Vec<MediaParameter>,
}

impl MediaType {
    pub(crate) fn new(media_range: MediaRange, parameters: Vec<MediaParameter>) -> Self {
        MediaType {
            media_range,
            parameters,
        }
    }

    /// Get a reference to the `MediaRange` of the media type.
    pub fn media_range(&self) -> &MediaRange {
        &self.media_range
    }

    /// Get a reference to the list of `MediaParameter`s of the media type.
    pub fn parameters(&self) -> &Vec<MediaParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.media_range,
            if self.parameters.is_empty() { "" } else { ";" },
            join(&self.parameters, ";")
        )
    }
}

impl PartialEq for MediaType {
    fn eq(&self, other: &Self) -> bool {
        if self.media_range != other.media_range {
            return false;
        }

        compare_vectors(self.parameters().deref(), other.parameters().deref())
    }
}

impl Hash for MediaType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.media_range.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

pub(crate) mod parser {
    use nom::{
        branch::alt,
        combinator::map,
        error::context,
        multi::many0,
        sequence::{preceded, separated_pair},
        Parser,
    };

    use crate::{
        common::media_range::parser::{m_subtype, m_type},
        common::wrapped_string::WrappedString,
        parser::{equal, quoted_string, semi, slash, token, ParserResult},
        MediaParameter, MediaRange, MediaType, TokenString,
    };

    #[inline]
    fn m_attribute(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }

    #[inline]
    fn m_value(input: &str) -> ParserResult<&str, WrappedString<TokenString>> {
        context(
            "m_value",
            alt((map(token, WrappedString::new_not_wrapped), quoted_string)),
        )
        .parse(input)
    }

    #[inline]
    fn m_parameter(input: &str) -> ParserResult<&str, MediaParameter> {
        context(
            "m_parameter",
            map(
                separated_pair(m_attribute, equal, m_value),
                |(key, value)| MediaParameter::new(key, value),
            ),
        )
        .parse(input)
    }

    pub(crate) fn media_type(input: &str) -> ParserResult<&str, MediaType> {
        context(
            "media_type",
            map(
                (m_type, slash, m_subtype, many0(preceded(semi, m_parameter))),
                |(r#type, _, subtype, parameters)| {
                    MediaType::new(MediaRange::new(r#type, subtype), parameters)
                },
            ),
        )
        .parse(input)
    }
}
