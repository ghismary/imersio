use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::hash::Hash;
use std::ops::Deref;

use crate::common::media_parameter::MediaParameter;
use crate::common::media_range::MediaRange;
use crate::utils::compare_vectors;

/// Representation of a media type contained in a `ContentTypeHeader`.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
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
