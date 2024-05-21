#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallIdHeader(String);

impl CallIdHeader {
    pub(crate) fn new(call_id: String) -> Self {
        CallIdHeader(call_id)
    }

    /// Get the call ID from the Call-ID header.
    pub fn call_id(&self) -> String {
        self.0.clone()
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
    use crate::Header;
    use std::str::FromStr;

    #[test]
    fn test_valid_call_id_header() {
        // Valid Call-ID header with `@` character
        let header = Header::from_str("Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com");
        assert!(header.is_ok());
        if let Header::CallId(header) = header.unwrap() {
            assert_eq!(
                header.call_id(),
                "f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com"
            );
        } else {
            panic!("Not an Call-ID header");
        }

        // Valid Call-ID header without `@` character
        let header = Header::from_str("Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6");
        assert!(header.is_ok());
        if let Header::CallId(header) = header.unwrap() {
            assert_eq!(header.call_id(), "f81d4fae-7dec-11d0-a765-00a0c91e6bf6");
        } else {
            panic!("Not an Call-ID header");
        }
    }

    #[test]
    fn test_invalid_call_id_header() {
        // Empty Call-ID header
        let header = Header::from_str("Call-ID:");
        assert!(header.is_err());

        // Empty Call-ID header with spaces
        let header = Header::from_str("Call-ID:    ");
        assert!(header.is_err());

        // Call-ID header with invalid character
        let header = Header::from_str("Call-ID: 😁");
        assert!(header.is_err());
    }

    #[test]
    fn test_call_id_header_equality() {
        let first_header = Header::from_str("Call-ID: a84b4c76e66710");
        let second_header = Header::from_str("Call-ID: a84b4c76e66710");
        if let (Header::CallId(first_header), Header::CallId(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Call-ID header");
        }
    }

    #[test]
    fn test_call_id_header_inequality() {
        let first_header =
            Header::from_str("Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com");
        let second_header = Header::from_str("Call-ID: a84b4c76e66710");
        if let (Header::CallId(first_header), Header::CallId(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Call-ID header");
        }

        let first_header =
            Header::from_str("Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com");
        let second_header = Header::from_str("Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6");
        if let (Header::CallId(first_header), Header::CallId(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Call-ID header");
        }
    }
}
