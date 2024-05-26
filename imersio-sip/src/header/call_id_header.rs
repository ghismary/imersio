#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallIdHeader(String);

impl CallIdHeader {
    pub(crate) fn new<S: Into<String>>(call_id: S) -> Self {
        CallIdHeader(call_id.into())
    }

    /// Get the call ID from the Call-ID header.
    pub fn call_id(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CallIdHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Call-ID: {}", self.0)
    }
}

impl PartialEq<&CallIdHeader> for CallIdHeader {
    fn eq(&self, other: &&CallIdHeader) -> bool {
        self == *other
    }
}

impl PartialEq<CallIdHeader> for &CallIdHeader {
    fn eq(&self, other: &CallIdHeader) -> bool {
        *self == other
    }
}

#[cfg(test)]
mod tests {
    use super::CallIdHeader;
    use crate::Header;
    use std::str::FromStr;

    fn valid_header<F: FnOnce(CallIdHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert!(header.is_ok());
        if let Header::CallId(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not a Call-ID header");
        }
    }

    #[test]
    fn test_valid_call_id_header_with_arobase_character() {
        valid_header(
            "Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com",
            |header| {
                assert_eq!(
                    header.call_id(),
                    "f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com"
                );
            },
        );
    }

    #[test]
    fn test_valid_call_id_header_without_arobase_character() {
        valid_header("Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6", |header| {
            assert_eq!(header.call_id(), "f81d4fae-7dec-11d0-a765-00a0c91e6bf6");
        });
    }

    fn invalid_header(header: &str) {
        assert!(Header::from_str(header).is_err());
    }

    #[test]
    fn test_invalid_call_id_header_empty() {
        invalid_header("Call-ID:");
    }

    #[test]
    fn test_invalid_call_id_header_empty_with_space_characters() {
        invalid_header("Call-ID:    ");
    }

    #[test]
    fn test_invalid_call_id_header_with_invalid_character() {
        invalid_header("Call-ID: 😁");
    }

    #[test]
    fn test_call_id_header_equality_same_header_with_space_characters_differences() {
        let first_header = Header::from_str("Call-ID: a84b4c76e66710");
        let second_header = Header::from_str("Call-ID:  a84b4c76e66710");
        if let (Header::CallId(first_header), Header::CallId(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not a Call-ID header");
        }
    }

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::CallId(first_header), Header::CallId(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not a Call-ID header");
        }
    }

    #[test]
    fn test_call_id_header_inequality_different_values() {
        header_inequality(
            "Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com",
            "Call-ID: a84b4c76e66710",
        );
    }

    #[test]
    fn test_call_id_header_inequality_one_with_arobase_part_the_other_without() {
        header_inequality(
            "Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com",
            "Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
        );
    }

    #[test]
    fn test_call_id_header_inequality_same_value_with_different_cases() {
        header_inequality("Call-ID: a84b4c76e66710", "Call-ID: A84B4C76E66710");
    }
}
