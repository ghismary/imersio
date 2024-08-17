//! SIP Alert-Info header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{Alert, Alerts};

/// Representation of an Alert-Info header.
///
/// When present in an INVITE request, the Alert-Info header field specifies
/// an alternative ringtone to the UAS. When present in a 180 (Ringing)
/// response, the Alert-Info header field specifies an alternative ringback
/// tone to the UAC. A typical usage is for a proxy to insert this header
/// field to provide a distinctive ring feature.
///
/// [[RFC3261, Section 20.4](https://datatracker.ietf.org/doc/html/rfc3261#section-20.4)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct AlertInfoHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    alerts: Alerts,
}

impl AlertInfoHeader {
    pub(crate) fn new(header: GenericHeader, alerts: Vec<Alert>) -> Self {
        Self {
            header,
            alerts: alerts.into(),
        }
    }

    /// Get a reference to the alerts from the `AlertInfoHeader`.
    pub fn alerts(&self) -> &Alerts {
        &self.alerts
    }
}

impl HeaderAccessor for AlertInfoHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Alert-Info")
    }
    fn normalized_value(&self) -> String {
        self.alerts.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::alert::parser::alert_param;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{AlertInfoHeader, Header};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn alert_info(input: &str) -> ParserResult<&str, Header> {
        context(
            "Alert-Info header",
            map(
                tuple((
                    tag_no_case("Alert-Info"),
                    hcolon,
                    cut(consumed(separated_list1(comma, alert_param))),
                )),
                |(name, separator, (value, alerts))| {
                    Header::AlertInfo(AlertInfoHeader::new(
                        GenericHeader::new(name, separator, value),
                        alerts,
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
        AlertInfoHeader, Header, Uri,
    };
    use claims::assert_ok;

    valid_header!(AlertInfo, AlertInfoHeader, "Alert-Info");
    header_equality!(AlertInfo, "Alert-Info");
    header_inequality!(AlertInfo, "Alert-Info");

    #[test]
    fn test_valid_alert_info_header() {
        valid_header(
            "Alert-Info: <http://www.example.com/sounds/moo.wav>",
            |header| {
                assert_eq!(header.alerts().len(), 1);
                assert!(header.alerts().contains(
                    Uri::try_from("http://www.example.com/sounds/moo.wav")
                        .unwrap()
                        .as_absolute_uri()
                        .unwrap()
                ));
            },
        );
    }

    #[test]
    fn test_invalid_alert_info_header_empty() {
        invalid_header("Alert-Info:");
    }

    #[test]
    fn test_invalid_alert_info_header_empty_with_space_characters() {
        invalid_header("Alert-Info:       ");
    }

    #[test]
    fn test_invalid_alert_info_header_missing_brackets_around_the_uri() {
        invalid_header("Alert-Info: http://www.example.com/sounds/moo.wav");
    }

    #[test]
    fn test_alert_info_header_equality_with_space_characters_differences() {
        header_equality(
            "Alert-Info: <http://www.example.com/sounds/moo.wav>",
            "Alert-Info:  <http://www.example.com/sounds/moo.wav>",
        );
    }

    #[test]
    fn test_alert_info_header_equality_with_same_uri_and_same_parameters_with_different_cases() {
        header_equality(
            "Alert-Info: <http://www.example.com/sounds/moo.wav>;myparam=test",
            "Alert-Info: <http://www.example.com/sounds/moo.wav> ;MyParam=TEST",
        );
    }

    #[test]
    fn test_alert_info_header_inequality_with_different_uris() {
        header_inequality(
            "Alert-Info: <http://www.example.com/sounds/moo.wav>",
            "Alert-Info: <http://www.example.com/sounds/bip.wav>",
        );
    }

    #[test]
    fn test_alert_info_header_inequality_with_same_uri_but_different_parameters() {
        header_inequality(
            "Alert-Info: <http://www.example.com/sounds/moo.wav>;foo=bar",
            "Alert-Info: <http://www.example.com/sounds/moo.wav>;foo=baz",
        );
    }

    #[test]
    fn test_alert_info_header_to_string() {
        let header = Header::try_from(
            "alert-info:   <http://www.example.com/sounds/moo.wav> ;    MyParam = TEST",
        );
        if let Header::AlertInfo(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "alert-info:   <http://www.example.com/sounds/moo.wav> ;    MyParam = TEST"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Alert-Info: <http://www.example.com/sounds/moo.wav>;myparam=test"
            );
            assert_eq!(
                header.to_compact_string(),
                "Alert-Info: <http://www.example.com/sounds/moo.wav>;myparam=test"
            );
        }
    }
}
