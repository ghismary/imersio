use crate::{FromParameter, FromParameters};

/// Representation of the list of from parameters of a `From` header.
///
/// This is usable as an iterator.
pub type ToParameters = FromParameters;

/// Representation of a parameter founded in a `To` header.
pub type ToParameter = FromParameter;
