use crate::{common::HeaderValueCollection, utils::partial_eq_refs, HeaderAccessor, Method};

use super::generic_header::GenericHeader;

/// Representation of an Allow header.
///
/// The Allow header field lists the set of methods supported by the UA
/// generating the message.
///
/// [[RFC3261, Section 20.5](https://datatracker.ietf.org/doc/html/rfc3261#section-20.5)]
#[derive(Clone, Debug, Eq)]
pub struct AllowHeader {
    header: GenericHeader,
    methods: Methods,
}

impl AllowHeader {
    pub(crate) fn new(header: GenericHeader, methods: Vec<Method>) -> Self {
        AllowHeader {
            header,
            methods: methods.into(),
        }
    }

    /// Get a reference to the list of methods from the Allow header.
    pub fn methods(&self) -> &Methods {
        &self.methods
    }

    /// Tell whether Allow header contains the given method.
    pub fn contains(&self, method: Method) -> bool {
        self.methods.iter().any(|m| m == method)
    }
}

impl HeaderAccessor for AllowHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Allow")
    }
    fn normalized_value(&self) -> String {
        self.methods.to_string()
    }
}

impl std::fmt::Display for AllowHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for AllowHeader {
    fn eq(&self, other: &Self) -> bool {
        self.methods == other.methods
    }
}

partial_eq_refs!(AllowHeader);

/// Representation of the list of methods from an `AllowHeader`.
///
/// This is usable as an iterator.
pub type Methods = HeaderValueCollection<Method>;

#[cfg(test)]
mod tests {
    use super::AllowHeader;
    use crate::{Header, HeaderAccessor, Method};
    use claim::assert_ok;
    use std::str::FromStr;

    fn valid_header<F: FnOnce(AllowHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert_ok!(&header);
        if let Header::Allow(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not an Allow header");
        }
    }

    #[test]
    fn test_valid_allow_header_with_methods() {
        valid_header("Allow: INVITE, ACK, OPTIONS, CANCEL, BYE", |header| {
            assert!(!header.methods().is_empty());
            assert_eq!(header.methods().len(), 5);
            assert!(header.contains(Method::INVITE));
            assert!(header.contains(Method::ACK));
            assert!(header.contains(Method::OPTIONS));
            assert!(header.contains(Method::CANCEL));
            assert!(header.contains(Method::BYE));
            assert!(!header.contains(Method::REGISTER));
        });
    }

    #[test]
    fn test_valid_allow_header_empty() {
        valid_header("Allow:", |header| {
            assert!(header.methods().is_empty());
            assert_eq!(header.methods().len(), 0);
            assert!(!header.contains(Method::INVITE));
            assert!(!header.contains(Method::REGISTER));
        });
    }

    #[test]
    fn test_valid_allow_header_empty_with_space_characters() {
        valid_header("Allow:      ", |header| {
            assert!(header.methods().is_empty());
            assert_eq!(header.methods().len(), 0);
            assert!(!header.contains(Method::CANCEL));
            assert!(!header.contains(Method::BYE));
        });
    }

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::Allow(first_header), Header::Allow(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Allow header");
        }
    }

    #[test]
    fn test_allow_header_equality_same_headers_with_space_characters_differences() {
        header_equality(
            "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE",
            "Allow:   INVITE, ACK,  OPTIONS, CANCEL,     BYE",
        );
    }

    #[test]
    fn test_allow_header_equality_with_different_methods_order() {
        header_equality(
            "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE",
            "Allow: INVITE, BYE, CANCEL, OPTIONS, ACK",
        );
    }

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::Allow(first_header), Header::Allow(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Allow header");
        }
    }

    #[test]
    fn test_allow_header_inequality_with_different_methods() {
        header_inequality("Allow: INVITE", "Allow: BYE");
    }

    #[test]
    fn test_allow_header_inequality_with_first_header_having_more_methods_than_the_second() {
        header_inequality(
            "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE",
            "Allow: BYE, CANCEL, REGISTER, ACK",
        );
    }

    #[test]
    fn test_allow_header_inequality_with_first_header_having_less_methods_than_the_second() {
        header_inequality("Allow: INVITE, ACK", "Allow: INVITE, BYE, CANCEL, ACK");
    }

    #[test]
    fn test_allow_header_inequality_with_non_uppercase_methods() {
        header_inequality(
            "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE",
            "allow: invite, Bye, CanCeL, OptionS, acK",
        );
    }

    #[test]
    fn test_allow_header_to_string() {
        let header = Header::from_str("allow:   INVITE , ACK,  OPTIONS   , CANCEL,     BYE");
        if let Header::Allow(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "allow:   INVITE , ACK,  OPTIONS   , CANCEL,     BYE"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE"
            );
            assert_eq!(
                header.to_compact_string(),
                "Allow: INVITE, ACK, OPTIONS, CANCEL, BYE"
            );
        }
    }
}
