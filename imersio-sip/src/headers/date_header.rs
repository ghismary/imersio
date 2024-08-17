//! SIP Date header parsing and generation.

use chrono::{DateTime, Utc};
use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};

/// Representation of a Date header.
///
/// The Date header field contains the date and time.
/// SIP only supports the most recent [RFC 1123](https://datatracker.ietf.org/doc/html/rfc1123).
/// SIP restricts the time zone in SIP-date to "GMT", while RFC 1123 allows any time zone.
///
/// [[RFC3261, Section 20.17](https://datatracker.ietf.org/doc/html/rfc3261#section-20.17)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
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
    crate::headers::generic_header_accessors!(header);

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

pub(crate) mod parser {
    use crate::headers::GenericHeader;
    use crate::parser::{digit, hcolon, sp, ParserResult};
    use crate::{DateHeader, Header};
    use chrono::{DateTime, Utc};
    use nom::{
        branch::alt,
        bytes::complete::{tag, tag_no_case},
        combinator::{consumed, cut, map, recognize},
        error::{context, ErrorKind, ParseError, VerboseError},
        multi::count,
        sequence::tuple,
    };

    fn wkday(input: &str) -> ParserResult<&str, &str> {
        context(
            "wkday",
            alt((
                tag("Mon"),
                tag("Tue"),
                tag("Wed"),
                tag("Thu"),
                tag("Fri"),
                tag("Sat"),
                tag("Sun"),
            )),
        )(input)
    }

    fn month(input: &str) -> ParserResult<&str, &str> {
        context(
            "month",
            alt((
                tag("Jan"),
                tag("Feb"),
                tag("Mar"),
                tag("Apr"),
                tag("May"),
                tag("Jun"),
                tag("Jul"),
                tag("Aug"),
                tag("Sep"),
                tag("Oct"),
                tag("Nov"),
                tag("Dec"),
            )),
        )(input)
    }

    fn date1(input: &str) -> ParserResult<&str, &str> {
        context(
            "date1",
            recognize(tuple((count(digit, 2), sp, month, sp, count(digit, 4)))),
        )(input)
    }

    fn time(input: &str) -> ParserResult<&str, &str> {
        context(
            "time",
            recognize(tuple((
                count(digit, 2),
                tag(":"),
                count(digit, 2),
                tag(":"),
                count(digit, 2),
            ))),
        )(input)
    }

    fn rfc1123_date(input: &str) -> ParserResult<&str, DateTime<Utc>> {
        let result = recognize(tuple((
            wkday,
            tag(","),
            sp,
            date1,
            sp,
            time,
            sp,
            tag("GMT"),
        )))(input);
        match result {
            Err(e) => Err(e),
            Ok((rest, date)) => {
                let result = DateTime::parse_from_rfc2822(date);
                match result {
                    Ok(date) => Ok((rest, date.to_utc())),
                    Err(_) => Err(nom::Err::Error(VerboseError::from_error_kind(
                        input,
                        ErrorKind::Verify,
                    ))),
                }
            }
        }
    }

    #[inline]
    fn sip_date(input: &str) -> ParserResult<&str, DateTime<Utc>> {
        rfc1123_date(input)
    }

    pub(crate) fn date(input: &str) -> ParserResult<&str, Header> {
        context(
            "Date header",
            map(
                tuple((tag_no_case("Date"), hcolon, cut(consumed(sip_date)))),
                |(name, separator, (value, date))| {
                    Header::Date(DateHeader::new(
                        GenericHeader::new(name, separator, value),
                        date,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
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
