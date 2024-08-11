#![allow(missing_docs)]

use derive_more::IsVariant;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

use crate::common::value_collection::ValueCollection;
use crate::Product;

/// Representation of the list of server values in a `Server` or `User-Agent` header.
///
/// This is usable as an iterator.
pub type ServerValues = ValueCollection<ServerValue>;

/// Representation of an server value contained in a `Server` or `User-Agent` header.
#[derive(Clone, Debug, Eq, Hash, IsVariant, PartialEq, PartialEqRefs)]
pub enum ServerValue {
    Product(Product),
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
    use crate::common::product::parser::product;
    use crate::parser::{comment, ParserResult};
    use crate::ServerValue;
    use nom::{branch::alt, combinator::map, error::context};

    pub(crate) fn server_val(input: &str) -> ParserResult<&str, ServerValue> {
        context(
            "server_val",
            alt((
                map(product, ServerValue::Product),
                map(comment, |comment| ServerValue::Comment(comment.to_string())),
            )),
        )(input)
    }
}