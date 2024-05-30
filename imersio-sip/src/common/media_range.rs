use partial_eq_refs::PartialEqRefs;

/// Representation of a media range contained in an `AcceptRange` or a
/// `Content-Type` header.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, PartialEqRefs)]
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

impl std::fmt::Display for MediaRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.r#type, self.subtype,)
    }
}
