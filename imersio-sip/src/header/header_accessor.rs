/// A trait defining the common accessors for all SIP message headers.
pub trait HeaderAccessor {
    /// Get the name of the header, as it has been parsed, whatever its case
    /// is.
    fn name(&self) -> &str;
    /// Get the separator of the header, as it has been parsed, containing
    /// any unnecessary space characters.
    fn separator(&self) -> &str;
    /// Get the value of the header, as it has been parsed, whatever its case
    /// is.
    fn value(&self) -> &str;

    /// Get the compact name of the header, if it has one.
    fn compact_name(&self) -> Option<&str>;
    /// Get the normalized name of the header, eg. `Call-ID`.
    fn normalized_name(&self) -> Option<&str>;
    /// Get the value of the header, cleaned from all unnecessary space
    /// characters and with case-insensitive parts converted to lowercase.
    fn normalized_value(&self) -> String;

    /// Tell whether the header has a compact name or not.
    fn has_compact_name(&self) -> bool {
        self.compact_name().is_some()
    }
    /// Get the compact format of the header.
    fn to_compact_string(&self) -> String {
        format!(
            "{}: {}",
            self.compact_name()
                .or_else(|| self.normalized_name())
                .or_else(|| self.name().into())
                .unwrap(),
            self.normalized_value()
        )
    }
    /// Get the normalized format of the header.
    fn to_normalized_string(&self) -> String {
        format!(
            "{}: {}",
            self.normalized_name()
                .or_else(|| self.name().into())
                .unwrap(),
            self.normalized_value()
        )
    }
}

macro_rules! generic_header_accessors {
    (
        $header:ident
    ) => {
        fn name(&self) -> &str {
            self.$header.name()
        }
        fn separator(&self) -> &str {
            self.$header.separator()
        }
        fn value(&self) -> &str {
            self.$header.value()
        }
    };
}
pub(crate) use generic_header_accessors;
