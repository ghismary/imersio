use crate::TokenString;
use std::hash::Hash;

/// Representation of a product, containing its name and version.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Product {
    name: TokenString,
    version: Option<TokenString>,
}

impl Product {
    pub(crate) fn new(name: TokenString, version: Option<TokenString>) -> Self {
        Product { name, version }
    }

    /// Get the name of the product.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the product.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref().map(|v| v.as_str())
    }
}

impl std::fmt::Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.name(),
            if self.version().is_some() { "/" } else { "" },
            self.version().unwrap_or_default()
        )
    }
}

pub(crate) mod parser {
    use nom::{
        combinator::{map, opt},
        error::context,
        sequence::{pair, preceded},
        Parser,
    };

    use crate::{
        parser::{slash, token, ParserResult},
        Product, TokenString,
    };

    pub(crate) fn product(input: &str) -> ParserResult<&str, Product> {
        context(
            "product",
            map(
                pair(token, opt(preceded(slash, product_version))),
                |(name, version)| Product::new(name, version),
            ),
        )
        .parse(input)
    }

    #[inline]
    fn product_version(input: &str) -> ParserResult<&str, TokenString> {
        token(input)
    }
}
