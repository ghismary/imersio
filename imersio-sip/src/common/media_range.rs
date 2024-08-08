use derive_more::Display;
use partial_eq_refs::PartialEqRefs;

/// Representation of a media range contained in an `AcceptRange` or a `Content-Type` header.
#[derive(Clone, Debug, Display, Eq, Hash, PartialEq, PartialEqRefs)]
#[display(fmt = "{}/{}", "self.r#type", "self.subtype")]
pub struct MediaRange {
    r#type: String,
    subtype: String,
}

impl MediaRange {
    pub(crate) fn new<S: Into<String>>(r#type: S, subtype: S) -> Self {
        MediaRange {
            r#type: r#type.into(),
            subtype: subtype.into(),
        }
    }
}

pub(crate) mod parser {
    use crate::parser::{slash, token, ParserResult};
    use crate::MediaRange;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{map, recognize},
        sequence::{pair, separated_pair},
    };

    fn discrete_type(input: &str) -> ParserResult<&str, &str> {
        alt((
            tag("text"),
            tag("image"),
            tag("audio"),
            tag("video"),
            tag("application"),
        ))(input)
    }

    fn composite_type(input: &str) -> ParserResult<&str, &str> {
        alt((tag("message"), tag("multipart")))(input)
    }

    #[inline]
    fn ietf_token(input: &str) -> ParserResult<&str, &str> {
        token(input)
    }

    fn x_token(input: &str) -> ParserResult<&str, &str> {
        recognize(pair(tag("x-"), token))(input)
    }

    fn extension_token(input: &str) -> ParserResult<&str, &str> {
        alt((ietf_token, x_token))(input)
    }

    pub(crate) fn m_type(input: &str) -> ParserResult<&str, &str> {
        alt((discrete_type, composite_type, extension_token))(input)
    }

    #[inline]
    fn iana_token(input: &str) -> ParserResult<&str, &str> {
        token(input)
    }

    pub(crate) fn m_subtype(input: &str) -> ParserResult<&str, &str> {
        alt((extension_token, iana_token))(input)
    }

    pub(crate) fn media_range(input: &str) -> ParserResult<&str, MediaRange> {
        map(
            alt((
                separated_pair(tag("*"), slash, tag("*")),
                separated_pair(m_type, slash, tag("*")),
                separated_pair(m_type, slash, m_subtype),
            )),
            |(r#type, subtype)| MediaRange::new(r#type, subtype),
        )(input)
    }
}
