use std::{collections::HashSet, hash::Hash};

use partial_eq_refs::PartialEqRefs;

use crate::{
    common::{accept_parameter::AcceptParameter, header_value_collection::HeaderValueCollection},
    AbsoluteUri, HeaderAccessor,
};

use super::generic_header::GenericHeader;

/// Representation of an Alert-Info header.
///
/// When present in an INVITE request, the Alert-Info header field specifies
/// an alternative ring tone to the UAS. When present in a 180 (Ringing)
/// response, the Alert-Info header field specifies an alternative ringback
/// tone to the UAC. A typical usage is for a proxy to insert this header
/// field to provide a distinctive ring feature.
///
/// [[RFC3261, Section 20.4](https://datatracker.ietf.org/doc/html/rfc3261#section-20.4)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct AlertInfoHeader {
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
    crate::header::generic_header_accessors!(header);

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

impl std::fmt::Display for AlertInfoHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for AlertInfoHeader {
    fn eq(&self, other: &Self) -> bool {
        self.alerts == other.alerts
    }
}

/// Representation of the list of alerts from an `AlertInfoHeader`.
///
/// This is usable as an iterator.
pub type Alerts = HeaderValueCollection<Alert>;

impl Alerts {
    /// Tell whether `Alerts` contain the given `Uri`.
    pub fn contains(&self, uri: &AbsoluteUri) -> bool {
        self.iter().any(|a| a.uri == uri)
    }

    /// Get the `Alert` corresponding to the given `Uri`.
    pub fn get(&self, uri: &AbsoluteUri) -> Option<&Alert> {
        self.iter().find(|a| a.uri == uri)
    }
}

/// Representation of an alert contained in an `Alert-Info` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct Alert {
    uri: AbsoluteUri,
    parameters: Vec<AcceptParameter>,
}

impl Alert {
    pub(crate) fn new(uri: AbsoluteUri, parameters: Vec<AcceptParameter>) -> Self {
        Alert { uri, parameters }
    }

    /// Get a reference to the uri contained in the `Alert`.
    pub fn uri(&self) -> &AbsoluteUri {
        &self.uri
    }

    /// Get a reference to the parameters contained in the `Alert`.
    pub fn parameters(&self) -> &Vec<AcceptParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for Alert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}>{}{}",
            self.uri,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl PartialEq for Alert {
    fn eq(&self, other: &Self) -> bool {
        if self.uri != other.uri {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

impl Hash for Alert {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::AlertInfoHeader;
    use crate::{
        header::tests::{header_equality, header_inequality, invalid_header, valid_header},
        Header, HeaderAccessor, Uri,
    };
    use claim::assert_ok;
    use std::str::FromStr;

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
                    Uri::from_str("http://www.example.com/sounds/moo.wav")
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
        let header = Header::from_str(
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
