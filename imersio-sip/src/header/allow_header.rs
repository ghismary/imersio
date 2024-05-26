use std::collections::HashSet;

use crate::Method;

#[derive(Clone, Debug, Eq)]
pub struct AllowHeader(Vec<Method>);

impl AllowHeader {
    pub(crate) fn new(methods: Vec<Method>) -> Self {
        AllowHeader(methods)
    }

    /// Tells whether the Allow header is empty or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of methods in the Allow header.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Tells whether Allow header contains the given method.
    pub fn contains(&self, method: Method) -> bool {
        self.0.iter().any(|m| m == method)
    }
}

impl std::fmt::Display for AllowHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Allow: {}",
            self.0
                .iter()
                .map(|method| method.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl PartialEq for AllowHeader {
    fn eq(&self, other: &Self) -> bool {
        let self_methods: HashSet<_> = self.0.iter().collect();
        let other_methods: HashSet<_> = other.0.iter().collect();
        self_methods == other_methods
    }
}

impl PartialEq<&AllowHeader> for AllowHeader {
    fn eq(&self, other: &&AllowHeader) -> bool {
        self == *other
    }
}

impl PartialEq<AllowHeader> for &AllowHeader {
    fn eq(&self, other: &AllowHeader) -> bool {
        *self == other
    }
}

#[cfg(test)]
mod tests {
    use super::AllowHeader;
    use crate::{Header, Method};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(AllowHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert!(header.is_ok());
        if let Header::Allow(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not an Allow header");
        }
    }

    #[test]
    fn test_valid_allow_header_with_methods() {
        valid_header("Allow: INVITE, ACK, OPTIONS, CANCEL, BYE", |header| {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 5);
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
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains(Method::INVITE));
            assert!(!header.contains(Method::REGISTER));
        });
    }

    #[test]
    fn test_valid_allow_header_empty_with_space_characters() {
        valid_header("Allow:      ", |header| {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
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
}
