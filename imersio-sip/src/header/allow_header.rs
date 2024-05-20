use std::collections::HashSet;

use crate::Method;

#[derive(Clone, Debug)]
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

impl Eq for AllowHeader {}

#[cfg(test)]
mod tests {
    use crate::{Header, Method};
    use std::str::FromStr;

    #[test]
    fn test_valid_allow_header() {
        // Valid Allow header
        let header = Header::from_str("Allow: INVITE, ACK, OPTIONS, CANCEL, BYE");
        assert!(header.is_ok());
        if let Header::Allow(header) = header.unwrap() {
            assert!(!header.is_empty());
            assert_eq!(header.count(), 5);
            assert!(header.contains(Method::INVITE));
            assert!(header.contains(Method::ACK));
            assert!(header.contains(Method::OPTIONS));
            assert!(header.contains(Method::CANCEL));
            assert!(header.contains(Method::BYE));
            assert!(!header.contains(Method::REGISTER));
        } else {
            panic!("Not an Allow header");
        }

        // Empty Allow header
        let header = Header::from_str("Allow:");
        assert!(header.is_ok());
        if let Header::Allow(header) = header.unwrap() {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains(Method::INVITE));
            assert!(!header.contains(Method::REGISTER));
        } else {
            panic!("Not an Allow header");
        }

        // Empty Allow header with space characters
        let header = Header::from_str("Allow:      ");
        assert!(header.is_ok());
        if let Header::Allow(header) = header.unwrap() {
            assert!(header.is_empty());
            assert_eq!(header.count(), 0);
            assert!(!header.contains(Method::CANCEL));
            assert!(!header.contains(Method::BYE));
        } else {
            panic!("Not an Allow header");
        }
    }

    #[test]
    fn test_allow_header_equality() {
        let first_header = Header::from_str("Allow: INVITE, ACK, OPTIONS, CANCEL, BYE");
        let second_header = Header::from_str("Allow: INVITE, BYE, CANCEL, OPTIONS, ACK");
        if let (Header::Allow(first_header), Header::Allow(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Allow header");
        }
    }

    #[test]
    fn test_allow_header_inequality() {
        let first_header = Header::from_str("Allow: INVITE, ACK, OPTIONS, CANCEL, BYE");
        let second_header = Header::from_str("Allow: BYE, CANCEL, REGISTER, ACK");
        if let (Header::Allow(first_header), Header::Allow(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Allow header");
        }

        let first_header = Header::from_str("Allow: INVITE, ACK");
        let second_header = Header::from_str("Allow: INVITE, BYE, CANCEL, ACK");
        if let (Header::Allow(first_header), Header::Allow(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Allow header");
        }
    }
}
