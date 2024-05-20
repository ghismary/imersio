use std::{collections::HashSet, hash::Hash};

use crate::Uri;

use super::accept_header::AcceptParameter;

#[derive(Clone, Debug)]
pub struct AlertInfoHeader(Vec<AlertParam>);

impl AlertInfoHeader {
    pub(crate) fn new(alerts: Vec<AlertParam>) -> Self {
        AlertInfoHeader(alerts)
    }

    /// Get the number of alerts in the Alert-Info header.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Tells whether Alert-Info header contains the given `Uri`.
    pub fn contains(&self, uri: &Uri) -> bool {
        self.0.iter().any(|a| a.uri == uri)
    }

    /// Gets the `AlertParam` corresponding to the given `Uri`.
    pub fn get(&self, uri: &Uri) -> Option<&AlertParam> {
        self.0.iter().find(|a| a.uri == uri)
    }
}

impl std::fmt::Display for AlertInfoHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
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

impl Eq for AlertInfoHeader {}

#[derive(Clone, Debug)]
pub struct AlertParam {
    uri: Uri,
    parameters: Vec<AcceptParameter>,
}

impl AlertParam {
    pub(crate) fn new(uri: Uri, parameters: Vec<AcceptParameter>) -> Self {
        AlertParam { uri, parameters }
    }
}

impl std::fmt::Display for AlertParam {
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

impl PartialEq for AlertParam {
    fn eq(&self, other: &Self) -> bool {
        if self.uri != other.uri {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

impl PartialEq<&AlertParam> for AlertParam {
    fn eq(&self, other: &&AlertParam) -> bool {
        self == *other
    }
}

impl PartialEq<AlertParam> for &AlertParam {
    fn eq(&self, other: &AlertParam) -> bool {
        *self == other
    }
}

impl Eq for AlertParam {}

impl Hash for AlertParam {
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
            assert!(
                header.contains(&Uri::from_str("http://www.example.com/sounds/moo.wav").unwrap())
            );
        } else {
            panic!("Not an Alert-Info header");
        }
    }

    #[test]
    fn test_invalid_alert_info_header() {
        // Empty Alert-Info header
        let header = Header::from_str("Alert-Info:");
        assert!(header.is_err());

        // Empty Alert-Info header with space characters
        let header = Header::from_str("Alert-Info:       ");
        assert!(header.is_err());

        // Missing brackets around the uri
        let header = Header::from_str("Alert-Info: http://www.example.com/sounds/moo.wav");
        assert!(header.is_err());
    }

    #[test]
    fn test_alert_info_header_equality() {
        let first_header = Header::from_str("Alert-Info: <http://www.example.com/sounds/moo.wav>");
        let second_header = Header::from_str("Alert-Info: <http://www.example.com/sounds/moo.wav>");
        if let (Header::AlertInfo(first_header), Header::AlertInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Alert-Info header");
        }
    }

    #[test]
    fn test_alert_info_header_inequality() {
        let first_header = Header::from_str("Alert-Info: <http://www.example.com/sounds/moo.wav>");
        let second_header = Header::from_str("Alert-Info: <http://www.example.com/sounds/bip.wav>");
        if let (Header::AlertInfo(first_header), Header::AlertInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Alert-Info header");
        }
    }
}
