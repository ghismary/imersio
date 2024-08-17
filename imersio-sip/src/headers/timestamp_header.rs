//! SIP Timestamp header parsing and generation.

use chrono::{DateTime, TimeDelta, Utc};
use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};

/// Representation of a Timestamp header.
///
/// The Timestamp header field describes when the UAC sent the request to the UAS.
///
/// [[RFC3261, Section 20.38](https://datatracker.ietf.org/doc/html/rfc3261#section-20.38)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct TimestampHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    timestamp: DateTime<Utc>,
    delay: Option<TimeDelta>,
}

impl TimestampHeader {
    pub(crate) fn new(
        header: GenericHeader,
        timestamp: DateTime<Utc>,
        delay: Option<TimeDelta>,
    ) -> Self {
        Self {
            header,
            timestamp,
            delay,
        }
    }

    /// Get a reference to the timestamp from the Timestamp header.
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    /// Get a reference to the delay from the Timestamp header.
    pub fn delay(&self) -> Option<&TimeDelta> {
        self.delay.as_ref()
    }
}

impl HeaderAccessor for TimestampHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Timestamp")
    }
    fn normalized_value(&self) -> String {
        let timestamp_seconds = self.timestamp().timestamp();
        let timestamp_nanoseconds = self.timestamp().timestamp_subsec_nanos();
        format!(
            "{}{}{}{}",
            timestamp_seconds,
            if timestamp_nanoseconds == 0 { "" } else { "." },
            if timestamp_nanoseconds == 0 {
                "".to_string()
            } else {
                timestamp_nanoseconds
                    .to_string()
                    .trim_end_matches('0')
                    .to_string()
            },
            match self.delay {
                Some(delay) => {
                    let delay_seconds = delay.num_seconds();
                    let delay_nanoseconds = delay.subsec_nanos();
                    format!(
                        " {}{}{}",
                        if delay_seconds == 0 {
                            "".to_string()
                        } else {
                            delay_seconds.to_string()
                        },
                        if delay_nanoseconds == 0 { "" } else { "." },
                        if delay_nanoseconds == 0 {
                            "".to_string()
                        } else {
                            delay_nanoseconds
                                .to_string()
                                .trim_end_matches('0')
                                .to_string()
                        }
                    )
                }
                None => "".to_string(),
            }
        )
    }
}

pub(crate) mod parser {
    use crate::headers::GenericHeader;
    use crate::parser::{digit, hcolon, lws, ParserResult};
    use crate::{Header, TimestampHeader};
    use chrono::{DateTime, TimeDelta};
    use nom::{
        bytes::complete::{tag, tag_no_case},
        character::complete::digit1,
        combinator::{consumed, cut, map, opt, recognize},
        error::context,
        multi::{many0, many_m_n},
        sequence::{pair, preceded, tuple},
    };

    pub(crate) fn timestamp(input: &str) -> ParserResult<&str, Header> {
        context(
            "Timestamp header",
            map(
                tuple((
                    tag_no_case("Timestamp"),
                    hcolon,
                    cut(consumed(tuple((
                        digit1,
                        opt(preceded(tag("."), recognize(many_m_n(0, 9, digit)))),
                        opt(preceded(lws, delay)),
                    )))),
                )),
                |(name, separator, (value, (seconds, nanoseconds, delay)))| {
                    Header::Timestamp(TimestampHeader::new(
                        GenericHeader::new(name, separator, value),
                        DateTime::from_timestamp(
                            seconds.parse::<i64>().unwrap_or(0),
                            super::str_to_nanoseconds(nanoseconds),
                        )
                        .unwrap()
                        .to_utc(),
                        delay,
                    ))
                },
            ),
        )(input)
    }

    fn delay(input: &str) -> ParserResult<&str, TimeDelta> {
        context(
            "delay",
            map(
                pair(
                    recognize(many0(digit)),
                    opt(preceded(tag("."), recognize(many_m_n(0, 9, digit)))),
                ),
                |(seconds, nanoseconds)| {
                    TimeDelta::new(
                        seconds.parse::<i64>().unwrap_or(0),
                        super::str_to_nanoseconds(nanoseconds),
                    )
                    .unwrap()
                },
            ),
        )(input)
    }
}

fn str_to_nanoseconds(value: Option<&str>) -> u32 {
    value
        .map(|v| {
            format!(
                "{}{}",
                v,
                (0..(9 - v.len())).map(|_| "0").collect::<String>()
            )
        })
        .unwrap_or("0".to_string())
        .parse::<u32>()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, TimestampHeader,
    };
    use chrono::{DateTime, TimeDelta};
    use claims::assert_ok;

    valid_header!(Timestamp, TimestampHeader, "Timestamp");
    header_equality!(Timestamp, "Timestamp");
    header_inequality!(Timestamp, "Timestamp");

    #[test]
    fn test_valid_timestamp_header_with_only_timestamp_seconds() {
        valid_header("Timestamp: 54", |header| {
            assert_eq!(
                header.timestamp(),
                &DateTime::from_timestamp(54, 0).unwrap().to_utc()
            );
        });
    }

    #[test]
    fn test_valid_timestamp_header_with_timestamp_seconds_and_nanoseconds() {
        valid_header("Timestamp: 837.387", |header| {
            assert_eq!(
                header.timestamp(),
                &DateTime::from_timestamp(837, 387_000_000).unwrap().to_utc()
            );
        });
    }

    #[test]
    fn test_valid_timestamp_header_with_seconds_delay() {
        valid_header("Timestamp: 138.752 3", |header| {
            assert_eq!(
                header.timestamp(),
                &DateTime::from_timestamp(138, 752_000_000).unwrap().to_utc()
            );
            assert_eq!(header.delay(), Some(&TimeDelta::new(3, 0).unwrap()))
        });
    }

    #[test]
    fn test_valid_timestamp_header_with_nanoseconds_delay() {
        valid_header("Timestamp: 138.35 .239", |header| {
            assert_eq!(
                header.timestamp(),
                &DateTime::from_timestamp(138, 350_000_000).unwrap().to_utc()
            );
            assert_eq!(
                header.delay(),
                Some(&TimeDelta::new(0, 239_000_000).unwrap())
            )
        });
    }

    #[test]
    fn test_valid_timestamp_header_with_seconds_and_nanoseconds_delay() {
        valid_header("Timestamp: 138.752 3.1", |header| {
            assert_eq!(
                header.timestamp(),
                &DateTime::from_timestamp(138, 752_000_000).unwrap().to_utc()
            );
            assert_eq!(
                header.delay(),
                Some(&TimeDelta::new(3, 100_000_000).unwrap())
            )
        });
    }

    #[test]
    fn test_invalid_timestamp_header_empty() {
        invalid_header("Timestamp:");
    }

    #[test]
    fn test_invalid_timestamp_header_empty_with_space_characters() {
        invalid_header("Timestamp:    ");
    }

    #[test]
    fn test_invalid_timestamp_header_with_invalid_character() {
        invalid_header("Timestamp: 😁");
    }

    #[test]
    fn test_timestamp_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            "Timestamp: 138.752 3.239",
            "Timestamp  :     138.752    3.239",
        );
    }

    #[test]
    fn test_timestamp_header_inequality_different_timestamp() {
        header_inequality("Timestamp: 138.752 3.239", "Timestamp: 54.399 3.239");
    }

    #[test]
    fn test_timestamp_header_inequality_different_delay() {
        header_inequality("Timestamp: 138.752 3.239", "Timestamp: 138.752 1.751");
    }

    #[test]
    fn test_timestamp_header_to_string() {
        let header = Header::try_from("tImEstAmp  :     138.752        3.239");
        if let Header::Timestamp(header) = header.unwrap() {
            assert_eq!(header.to_string(), "tImEstAmp  :     138.752        3.239");
            assert_eq!(header.to_normalized_string(), "Timestamp: 138.752 3.239");
            assert_eq!(header.to_compact_string(), "Timestamp: 138.752 3.239");
        }
    }
}
