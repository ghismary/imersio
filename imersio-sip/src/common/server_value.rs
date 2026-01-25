use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::Product;

/// Representation of the list of server values in a `Server` or `User-Agent` header.
///
/// This is usable as an iterator.
pub type ServerValues = ValueCollection<ServerValue>;

/// Representation of an server value contained in a `Server` or `User-Agent` header.
#[derive(Clone, Debug, Eq, Hash, PartialEq, derive_more::IsVariant)]
pub enum ServerValue {
    /// A product name and version
    Product(Product),
    /// A comment.
    Comment(String),
}

impl std::fmt::Display for ServerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Product(product) => write!(f, "{}", product),
            Self::Comment(comment) => write!(f, "{}", comment),
        }
    }
}

pub(crate) mod parser {
    use nom::{branch::alt, combinator::map, error::context, Parser};

    use crate::{
        common::product::parser::product,
        parser::{comment, ParserResult},
        ServerValue,
    };

    pub(crate) fn server_val(input: &str) -> ParserResult<&str, ServerValue> {
        context(
            "server_val",
            alt((
                map(product, ServerValue::Product),
                map(comment, |comment| ServerValue::Comment(comment.to_string())),
            )),
        )
        .parse(input)
    }
}
