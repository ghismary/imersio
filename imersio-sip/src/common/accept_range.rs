use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;

use crate::common::header_value_collection::HeaderValueCollection;
use crate::utils::compare_vectors;
use crate::AcceptParameter;
use crate::MediaRange;

/// Representation of the list of range from an `AcceptHeader`.
///
/// This is usable as an iterator.
pub type AcceptRanges = HeaderValueCollection<AcceptRange>;

impl AcceptRanges {
    /// Tell whether the ranges contain the given `MediaRange`.
    pub fn contains(&self, media_range: &MediaRange) -> bool {
        self.iter().any(|ar| ar.media_range == media_range)
    }

    /// Get the `Accept-Range` corresponding to the given `MediaRange`.
    pub fn get(&self, media_range: &MediaRange) -> Option<&AcceptRange> {
        self.iter().find(|ar| ar.media_range == media_range)
    }
}

/// Representation of a range contained in an `AcceptHeader`.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct AcceptRange {
    media_range: MediaRange,
    parameters: Vec<AcceptParameter>,
}

impl AcceptRange {
    pub(crate) fn new(media_range: MediaRange, parameters: Vec<AcceptParameter>) -> Self {
        AcceptRange {
            media_range,
            parameters,
        }
    }

    /// Get a reference to the `MediaRange` of the `AcceptRange`.
    pub fn media_range(&self) -> &MediaRange {
        &self.media_range
    }

    /// Get a reference to the vector of `AcceptParameter` of the `AcceptRange`.
    pub fn parameters(&self) -> &Vec<AcceptParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for AcceptRange {
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

impl PartialEq for AcceptRange {
    fn eq(&self, other: &Self) -> bool {
        self.media_range == other.media_range
            && compare_vectors(self.parameters(), other.parameters())
    }
}

impl Hash for AcceptRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.media_range.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

pub(crate) mod parser {
    use crate::common::accept_parameter::parser::accept_param;
    use crate::common::media_range::parser::media_range;
    use crate::parser::{semi, ParserResult};
    use crate::AcceptRange;
    use nom::{
        combinator::map,
        error::context,
        multi::many0,
        sequence::{pair, preceded},
    };

    pub(crate) fn accept_range(input: &str) -> ParserResult<&str, AcceptRange> {
        context(
            "accept_range",
            map(
                pair(media_range, many0(preceded(semi, accept_param))),
                |(media_range, accept_params)| AcceptRange::new(media_range, accept_params),
            ),
        )(input)
    }
}
