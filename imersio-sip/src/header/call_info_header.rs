use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::{uri::AbsoluteUri, GenericParameter};

#[derive(Clone, Debug, Eq)]
pub struct CallInfoHeader(Vec<CallInfo>);

impl CallInfoHeader {
    pub(crate) fn new(infos: Vec<CallInfo>) -> Self {
        CallInfoHeader(infos)
    }
}

impl CallInfoHeader {
    /// Get the number of infos in the Call-Info header.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Tells whether Call-Info header contains the given `AbsoluteUri`.
    pub fn contains(&self, uri: &AbsoluteUri) -> bool {
        self.0.iter().any(|info| info.uri == uri)
    }

    /// Gets the `CallInfo` corresponding to the given `AbsoluteUri`.
    pub fn get(&self, uri: &AbsoluteUri) -> Option<&CallInfo> {
        self.0.iter().find(|info| info.uri == uri)
    }
}

impl std::fmt::Display for CallInfoHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Call-Info: {}",
            self.0
                .iter()
                .map(|info| info.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl PartialEq for CallInfoHeader {
    fn eq(&self, other: &Self) -> bool {
        let self_call_infos: HashSet<_> = self.0.iter().collect();
        let other_call_infos: HashSet<_> = other.0.iter().collect();
        self_call_infos == other_call_infos
    }
}

impl PartialEq<&CallInfoHeader> for CallInfoHeader {
    fn eq(&self, other: &&CallInfoHeader) -> bool {
        self == *other
    }
}

impl PartialEq<CallInfoHeader> for &CallInfoHeader {
    fn eq(&self, other: &CallInfoHeader) -> bool {
        *self == other
    }
}

#[derive(Clone, Debug, Eq)]
pub struct CallInfo {
    uri: AbsoluteUri,
    parameters: Vec<CallInfoParameter>,
}

impl CallInfo {
    pub(crate) fn new(uri: AbsoluteUri, parameters: Vec<CallInfoParameter>) -> Self {
        CallInfo { uri, parameters }
    }

    /// Get a reference to the uri of the `CallInfo`.
    pub fn uri(&self) -> &AbsoluteUri {
        &self.uri
    }

    /// Get a reference to the parameters of the `CallInfo`.
    pub fn parameters(&self) -> &Vec<CallInfoParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for CallInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
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

impl PartialEq for CallInfo {
    fn eq(&self, other: &Self) -> bool {
        if self.uri != other.uri {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

impl PartialEq<&CallInfo> for CallInfo {
    fn eq(&self, other: &&CallInfo) -> bool {
        self == *other
    }
}

impl PartialEq<CallInfo> for &CallInfo {
    fn eq(&self, other: &CallInfo) -> bool {
        *self == other
    }
}

impl Hash for CallInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[derive(Clone, Debug, Eq)]
pub enum CallInfoParameter {
    IconPurpose,
    InfoPurpose,
    CardPurpose,
    OtherPurpose(String),
    Other(GenericParameter),
}

impl CallInfoParameter {
    pub(crate) fn new<S: Into<String>>(key: S, value: Option<S>) -> Self {
        match (
            key.into().to_ascii_lowercase().as_str(),
            value.map(|v| v.into().to_ascii_lowercase()).as_deref(),
        ) {
            ("purpose", Some("icon")) => Self::IconPurpose,
            ("purpose", Some("info")) => Self::InfoPurpose,
            ("purpose", Some("card")) => Self::CardPurpose,
            ("purpose", Some(value)) => Self::OtherPurpose(value.to_string()),
            (key, value) => Self::Other(GenericParameter::new(key, value)),
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Self::IconPurpose | Self::InfoPurpose | Self::CardPurpose | Self::OtherPurpose(_) => {
                "purpose"
            }
            Self::Other(value) => value.key(),
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            Self::IconPurpose => Some("icon"),
            Self::InfoPurpose => Some("info"),
            Self::CardPurpose => Some("card"),
            Self::OtherPurpose(value) => Some(value),
            Self::Other(value) => value.value(),
        }
    }
}

impl std::fmt::Display for CallInfoParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.key(),
            if self.value().is_some() { "=" } else { "" },
            self.value().unwrap_or_default()
        )
    }
}

impl PartialEq<CallInfoParameter> for CallInfoParameter {
    fn eq(&self, other: &CallInfoParameter) -> bool {
        match (self, other) {
            (Self::IconPurpose, Self::IconPurpose)
            | (Self::InfoPurpose, Self::InfoPurpose)
            | (Self::CardPurpose, Self::CardPurpose) => true,
            (Self::OtherPurpose(svalue), Self::OtherPurpose(ovalue)) => {
                svalue.eq_ignore_ascii_case(ovalue)
            }
            (Self::Other(a), Self::Other(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialEq<&CallInfoParameter> for CallInfoParameter {
    fn eq(&self, other: &&CallInfoParameter) -> bool {
        self == *other
    }
}

impl PartialEq<CallInfoParameter> for &CallInfoParameter {
    fn eq(&self, other: &CallInfoParameter) -> bool {
        *self == other
    }
}

impl Hash for CallInfoParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state);
        self.value().hash(state);
    }
}

impl PartialOrd for CallInfoParameter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CallInfoParameter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
    }
}

impl From<GenericParameter> for CallInfoParameter {
    fn from(value: GenericParameter) -> Self {
        CallInfoParameter::new(value.key(), value.value())
    }
}

#[cfg(test)]
mod tests {
    use super::CallInfoHeader;
    use crate::{header::call_info_header::CallInfoParameter, GenericParameter, Header, Uri};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(CallInfoHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert!(header.is_ok());
        if let Header::CallInfo(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not a Call-Info header");
        }
    }

    #[test]
    fn test_valid_call_info_header_with_icon_and_info() {
        valid_header("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info", |header| {
            assert_eq!(header.count(), 2);
            let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
            let first_uri = first_uri.as_absolute_uri().unwrap();
            assert!(header.contains(first_uri));
            let first_call_info = header.get(first_uri);
            assert!(first_call_info.is_some());
            let first_call_info = first_call_info.unwrap();
            assert_eq!(first_call_info.parameters().len(), 1);
            assert_eq!(
                first_call_info.parameters().first().unwrap(),
                CallInfoParameter::IconPurpose
            );
            let second_uri = Uri::from_str("http://www.example.com/alice/").unwrap();
            let second_uri = second_uri.as_absolute_uri().unwrap();
            assert!(header.contains(second_uri));
            let second_call_info = header.get(second_uri);
            assert!(second_call_info.is_some());
            let second_call_info = second_call_info.unwrap();
            assert_eq!(second_call_info.parameters().len(), 1);
            assert_eq!(
                second_call_info.parameters().first().unwrap(),
                CallInfoParameter::InfoPurpose
            );
            let third_uri = Uri::from_str("http://www.example.com/bob/").unwrap();
            let third_uri = third_uri.as_absolute_uri().unwrap();
            assert!(!header.contains(third_uri));
        });
    }

    #[test]
    fn test_valid_call_info_header_with_custom_purpose() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=photo",
            |header| {
                assert_eq!(header.count(), 1);
                let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.contains(first_uri));
                let first_call_info = header.get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert_eq!(first_call_info.parameters().len(), 1);
                assert_eq!(
                    first_call_info.parameters().first().unwrap(),
                    CallInfoParameter::OtherPurpose("photo".to_string())
                );
            },
        );
    }

    #[test]
    fn test_valid_call_info_header_with_custom_param_with_value() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;info=photo",
            |header| {
                assert_eq!(header.count(), 1);
                let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.contains(first_uri));
                let first_call_info = header.get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert_eq!(first_call_info.parameters().len(), 1);
                assert_eq!(
                    first_call_info.parameters().first().unwrap(),
                    CallInfoParameter::Other(GenericParameter::new("info", Some("photo")))
                );
            },
        );
    }

    #[test]
    fn test_valid_call_info_header_with_custom_param_without_value() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;info",
            |header| {
                assert_eq!(header.count(), 1);
                let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.contains(first_uri));
                let first_call_info = header.get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert_eq!(first_call_info.parameters().len(), 1);
                assert_eq!(
                    first_call_info.parameters().first().unwrap(),
                    CallInfoParameter::Other(GenericParameter::new("info", None))
                );
            },
        );
    }

    #[test]
    fn test_valid_call_info_header_without_param() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg>",
            |header| {
                assert_eq!(header.count(), 1);
                let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.contains(first_uri));
                let first_call_info = header.get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert!(first_call_info.parameters().is_empty());
            },
        );
    }

    fn invalid_header(header: &str) {
        assert!(Header::from_str(header).is_err());
    }

    #[test]
    fn test_invalid_call_info_header_empty() {
        invalid_header("Call-Info:");
    }

    #[test]
    fn test_invalid_call_info_header_empty_with_space_characters() {
        invalid_header("Call-Info:    ");
    }

    #[test]
    fn test_invalid_call_info_header_with_invalid_character() {
        invalid_header("Call-Info: 😁");
    }

    #[test]
    fn test_invalid_call_info_header_with_invalid_uri() {
        invalid_header("Call-Info: http://wwww.example.com/alice/photo.jpg");
    }

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::CallInfo(first_header), Header::CallInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Authorization header");
        }
    }

    #[test]
    fn test_call_info_header_equality_same_header_with_space_characters_differences() {
        header_equality("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/>;purpose=info", "Call-Info: <http://wwww.example.com/alice/photo.jpg>; purpose=icon, <http://www.example.com/alice/> ;purpose=info");
    }

    #[test]
    fn test_call_info_header_equality_with_inverted_infos() {
        header_equality("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info", "Call-Info: <http://www.example.com/alice/> ;purpose=info, <http://wwww.example.com/alice/photo.jpg> ;purpose=icon");
    }

    #[test]
    fn test_call_info_header_equality_with_different_cases() {
        header_equality(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon",
            "call-info: <http://wwww.example.com/alice/photo.jpg> ;puRpoSe=Icon",
        );
    }

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (Header::CallInfo(first_header), Header::CallInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not a Call-Info header");
        }
    }

    #[test]
    fn test_call_info_header_inequality_different_uris_with_same_purpose() {
        header_inequality(
            "Call-Info: <http://www.example.com/alice/> ;purpose=info",
            "Call-Info: <http://www.example.com/bob/> ;purpose=info",
        );
    }

    #[test]
    fn test_call_info_header_inequality_same_uri_with_different_purposes() {
        header_inequality(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon",
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=info",
        );
    }
}
