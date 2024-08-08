use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::utils::compare_vectors;
use crate::GenericParameter;
use crate::NameAddress;

/// Representation of the list of routes from a `RecordRouteHeader`.
///
/// This is usable as an iterator.
pub type Routes = HeaderValueCollection<Route>;

impl Routes {
    /// Tell whether the routes contain the given `NameAddress`.
    pub fn contains(&self, name_addr: &NameAddress) -> bool {
        self.iter().any(|r| r.name_addr == name_addr)
    }

    /// Get the route corresponding to the given `NameAddress`.
    pub fn get(&self, name_addr: &NameAddress) -> Option<&Route> {
        self.iter().find(|r| r.name_addr == name_addr)
    }
}

/// Representation of a route contained in a `Record-Route` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct Route {
    name_addr: NameAddress,
    parameters: Vec<GenericParameter>,
}

impl Route {
    pub(crate) fn new(name_addr: NameAddress, parameters: Vec<GenericParameter>) -> Self {
        Route {
            name_addr,
            parameters,
        }
    }

    /// Get a reference to the `NameAddress` contained in the route.
    pub fn name_address(&self) -> &NameAddress {
        &self.name_addr
    }

    /// Get a reference to the parameters contained in the route.
    pub fn parameters(&self) -> &Vec<GenericParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.name_addr,
            if self.parameters.is_empty() { "" } else { ";" },
            join(&self.parameters, ";")
        )
    }
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.name_addr == other.name_addr && compare_vectors(self.parameters(), other.parameters())
    }
}

impl Hash for Route {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name_addr.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

pub(crate) mod parser {
    use crate::common::contact::parser::name_addr;
    use crate::common::generic_parameter::parser::generic_param;
    use crate::parser::ParserResult;
    use crate::{GenericParameter, Route};
    use nom::{combinator::map, error::context, multi::many0, sequence::pair};

    #[inline]
    fn route_param(input: &str) -> ParserResult<&str, GenericParameter> {
        generic_param(input)
    }

    pub(crate) fn route(input: &str) -> ParserResult<&str, Route> {
        context(
            "route",
            map(
                pair(name_addr, many0(route_param)),
                |(name_addr, params)| Route::new(name_addr, params),
            ),
        )(input)
    }
}
