use crate::TokenString;
use derive_more::Display;

/// Representation of a media range contained in an `AcceptRange` or a `Content-Type` header.
#[derive(Clone, Debug, Display, Eq, Hash, PartialEq)]
#[display("{}/{}", self.r#type, self.subtype)]
pub struct MediaRange {
    r#type: TokenString,
    subtype: TokenString,
}

impl MediaRange {
    pub(crate) fn new(r#type: TokenString, subtype: TokenString) -> Self {
        MediaRange { r#type, subtype }
    }
}

pub(crate) mod parser {
    use crate::parser::{slash, token, ParserResult};
    use crate::{MediaRange, TokenString};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, recognize},
        sequence::{pair, separated_pair},
    };

    fn discrete_type(input: &str) -> ParserResult<&str, TokenString> {
        map(
            alt((
                tag("text"),
                tag("image"),
                tag("audio"),
                tag("video"),
                tag("application"),
            )),
            TokenString::new,
        )(input)
    }

    fn composite_type(input: &str) -> ParserResult<&str, TokenString> {
        map(alt((tag("message"), tag("multipart"))), TokenString::new)(input)
    }

    #[inline]
    fn ietf_token(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }

    #[inline]
    fn x_token(input: &str) -> ParserResult<&str, TokenString> {
        map(recognize(pair(tag("x-"), token)), TokenString::new)(input)
    }

    #[inline]
    fn extension_token(input: &str) -> ParserResult<&str, TokenString> {
        alt((ietf_token, x_token))(input)
    }

    pub(crate) fn m_type(input: &str) -> ParserResult<&str, TokenString> {
        alt((discrete_type, composite_type, extension_token))(input)
    }

    #[inline]
    fn iana_token(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }

    pub(crate) fn m_subtype(input: &str) -> ParserResult<&str, TokenString> {
        alt((extension_token, iana_token))(input)
    }

    pub(crate) fn media_range(input: &str) -> ParserResult<&str, MediaRange> {
        map(
            alt((
                separated_pair(
                    map(tag("*"), TokenString::new),
                    slash,
                    map(tag("*"), TokenString::new),
                ),
                separated_pair(m_type, slash, map(tag("*"), TokenString::new)),
                separated_pair(m_type, slash, m_subtype),
            )),
            |(r#type, subtype)| MediaRange::new(r#type, subtype),
        )(input)
    }
}
