//! SIP Date header parsing and generation.

use chrono::{DateTime, Utc};
use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::header::{GenericHeader, HeaderAccessor};

/// Representation of a Date header.
///
/// The Date header field contains the date and time.
/// SIP only supports the most recent [RFC 1123](https://datatracker.ietf.org/doc/html/rfc1123).
/// SIP restricts the time zone in SIP-date to "GMT", while RFC 1123 allows any time zone.
///
/// [[RFC3261, Section 20.17](https://datatracker.ietf.org/doc/html/rfc3261#section-20.17)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct DateHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    datetime: DateTime<Utc>,
}

impl DateHeader {
    pub(crate) fn new(header: GenericHeader, datetime: DateTime<Utc>) -> Self {
        Self { header, datetime }
    }

    /// Get the date and time from the Date header.
    pub fn datetime(&self) -> &DateTime<Utc> {
        &self.datetime
    }
}

impl HeaderAccessor for DateHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Date")
    }
    fn normalized_value(&self) -> String {
        self.datetime.format("%a, %d %b %Y %T GMT").to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        header::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        DateHeader, Header,
    };
    use chrono::prelude::*;
    use claims::assert_ok;

    valid_header!(Date, DateHeader, "Date");
    header_equality!(Date, "Date");
    header_inequality!(Date, "Date");

    #[test]
    fn test_valid_date_header() {
        valid_header("Date: Sat, 13 Nov 2010 23:29:00 GMT", |header| {
            assert_eq!(
                *header.datetime(),
                Utc.with_ymd_and_hms(2010, 11, 13, 23, 29, 0).unwrap()
            );
        });
    }

    #[test]
    fn test_invalid_date_header_empty() {
        invalid_header("Date:");
    }

    #[test]
    fn test_invalid_date_header_empty_with_space_characters() {
        invalid_header("Date:    ");
    }

    #[test]
    fn test_invalid_date_header_with_invalid_character() {
        invalid_header("Date: 😁");
    }

    #[test]
    fn test_invalid_date_header_wrong_case() {
        invalid_header("Date: sat, 13 nov 2010 23:29:00 Gmt");
    }

    #[test]
    fn test_date_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            "Date: Thu, 21 Feb 2002 13:02:03 GMT",
            "Date:    Thu, 21 Feb 2002 13:02:03 GMT",
        );
    }

    #[test]
    fn test_date_header_inequality_different_dates() {
        header_inequality(
            "Date: Thu, 21 Feb 2002 13:02:03 GMT",
            "Date: Sat, 13 Nov 2010 23:29:00 GMT",
        );
    }

    #[test]
    fn test_date_header_to_string() {
        let header = Header::try_from("date:    Thu, 21 Feb 2002 13:02:03 GMT");
        if let Header::Date(header) = header.unwrap() {
            assert_eq!(header.to_string(), "date:    Thu, 21 Feb 2002 13:02:03 GMT");
            assert_eq!(
                header.to_normalized_string(),
                "Date: Thu, 21 Feb 2002 13:02:03 GMT"
            );
            assert_eq!(
                header.to_compact_string(),
                "Date: Thu, 21 Feb 2002 13:02:03 GMT"
            );
        }
    }
}
