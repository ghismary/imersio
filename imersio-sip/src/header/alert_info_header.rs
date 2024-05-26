use std::{collections::HashSet, hash::Hash};

use crate::{common::AcceptParameter, uri::AbsoluteUri};

#[derive(Clone, Debug, Eq)]
pub struct AlertInfoHeader(Vec<AlertParameter>);

impl AlertInfoHeader {
    pub(crate) fn new(alerts: Vec<AlertParameter>) -> Self {
        AlertInfoHeader(alerts)
    }

    /// Get the number of alerts in the Alert-Info header.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Tells whether Alert-Info header contains the given `Uri`.
    pub fn contains(&self, uri: &AbsoluteUri) -> bool {
        self.0.iter().any(|a| a.uri == uri)
    }

    /// Gets the `AlertParam` corresponding to the given `Uri`.
    pub fn get(&self, uri: &AbsoluteUri) -> Option<&AlertParameter> {
        self.0.iter().find(|a| a.uri == uri)
    }
}

impl std::fmt::Display for AlertInfoHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Alert-Info: {}",
            self.0
                .iter()
                .map(|alert| alert.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl PartialEq for AlertInfoHeader {
    fn eq(&self, other: &Self) -> bool {
        let self_alerts: HashSet<_> = self.0.iter().collect();
        let other_alerts: HashSet<_> = other.0.iter().collect();
        self_alerts == other_alerts
    }
}

impl PartialEq<&AlertInfoHeader> for AlertInfoHeader {
    fn eq(&self, other: &&AlertInfoHeader) -> bool {
        self == *other
    }
}

impl PartialEq<AlertInfoHeader> for &AlertInfoHeader {
    fn eq(&self, other: &AlertInfoHeader) -> bool {
        *self == other
    }
}

#[derive(Clone, Debug, Eq)]
pub struct AlertParameter {
    uri: AbsoluteUri,
    parameters: Vec<AcceptParameter>,
}

impl AlertParameter {
    pub(crate) fn new(uri: AbsoluteUri, parameters: Vec<AcceptParameter>) -> Self {
        AlertParameter { uri, parameters }
    }
}

impl std::fmt::Display for AlertParameter {
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

impl PartialEq for AlertParameter {
    fn eq(&self, other: &Self) -> bool {
        if self.uri != other.uri {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

impl PartialEq<&AlertParameter> for AlertParameter {
    fn eq(&self, other: &&AlertParameter) -> bool {
        self == *other
    }
}

impl PartialEq<AlertParameter> for &AlertParameter {
    fn eq(&self, other: &AlertParameter) -> bool {
        *self == other
    }
}

impl Hash for AlertParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Header, Uri};
    use std::str::FromStr;

    #[test]
    fn test_valid_alert_info_header() {
        let header = Header::from_str("Alert-Info: <http://www.example.com/sounds/moo.wav>");
        assert!(header.is_ok());
        if let Header::AlertInfo(header) = header.unwrap() {
            assert_eq!(header.count(), 1);
            assert!(header.contains(
                Uri::from_str("http://www.example.com/sounds/moo.wav")
                    .unwrap()
                    .as_absolute_uri()
                    .unwrap()
            ));
        } else {
            panic!("Not an Alert-Info header");
        }
    }

    fn invalid_header(header: &str) {
        assert!(Header::from_str(header).is_err());
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

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::AlertInfo(first_header), Header::AlertInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Alert-Info header");
        }
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

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::AlertInfo(first_header), Header::AlertInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Alert-Info header");
        }
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
}
