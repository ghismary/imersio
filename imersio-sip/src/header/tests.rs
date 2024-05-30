use std::str::FromStr;

use claim::assert_err;

use crate::Header;

macro_rules! valid_header {
    (
        $enum:ident, $header:ident, $name:literal
    ) => {
        fn valid_header<F: FnOnce($header)>(header: &str, f: F) {
            let header = Header::from_str(header);
            assert_ok!(&header);
            if let Header::$enum(header) = header.unwrap() {
                f(header);
            } else {
                panic!("Not a {} header", $name);
            }
        }
    };
}
pub(crate) use valid_header;

pub(crate) fn invalid_header(header: &str) {
    assert_err!(Header::from_str(header));
}

macro_rules! header_equality {
    (
        $enum:ident, $name:literal
    ) => {
        fn header_equality(first_header: &str, second_header: &str) {
            let first_header = Header::from_str(first_header);
            let second_header = Header::from_str(second_header);
            if let (Header::$enum(first_header), Header::$enum(second_header)) =
                (first_header.unwrap(), second_header.unwrap())
            {
                assert_eq!(first_header, second_header);
            } else {
                panic!("Not a {} header", $name);
            }
        }
    };
}
pub(crate) use header_equality;

macro_rules! header_inequality {
    (
        $enum:ident, $name:literal
    ) => {
        fn header_inequality(first_header: &str, second_header: &str) {
            let first_header = Header::from_str(first_header);
            let second_header = Header::from_str(second_header);
            if let (Header::$enum(first_header), Header::$enum(second_header)) =
                (first_header.unwrap(), second_header.unwrap())
            {
                assert_ne!(first_header, second_header);
            } else {
                panic!("Not a {} header", $name);
            }
        }
    };
}
pub(crate) use header_inequality;
