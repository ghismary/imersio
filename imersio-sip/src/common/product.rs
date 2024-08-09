use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

/// Representation of a product, containing its name and version.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialEqRefs)]
pub struct Product {
    name: String,
    version: Option<String>,
}

impl Product {
    pub(crate) fn new<S: Into<String>>(name: S, version: Option<S>) -> Self {
        Product {
            name: name.into(),
            version: version.map(Into::into),
        }
    }

    /// Get the name of the product.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the product.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
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
    use crate::parser::{slash, token, ParserResult};
    use crate::Product;
    use nom::{
        combinator::{map, opt},
        error::context,
        sequence::{pair, preceded},
    };

    pub(crate) fn product(input: &str) -> ParserResult<&str, Product> {
        context(
            "product",
            map(
                pair(token, opt(preceded(slash, product_version))),
                |(name, version)| Product::new(name, version),
            ),
        )(input)
    }

    #[inline]
    fn product_version(input: &str) -> ParserResult<&str, &str> {
        token(input)
    }
}
