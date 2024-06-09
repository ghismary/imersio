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
