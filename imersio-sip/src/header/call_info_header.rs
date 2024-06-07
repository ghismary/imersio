use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use partial_eq_refs::PartialEqRefs;

use crate::{
    common::header_value_collection::HeaderValueCollection, AbsoluteUri, GenericParameter,
};

use super::{generic_header::GenericHeader, HeaderAccessor};

/// Representation of a Call-Info header.
///
/// The Call-Info header field provides additional information about the
/// caller or callee, depending on whether it is found in a request or
/// response.
///
/// [[RFC3261, Section 20.9](https://datatracker.ietf.org/doc/html/rfc3261#section-20.9)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct CallInfoHeader {
    header: GenericHeader,
    infos: CallInfos,
}

impl CallInfoHeader {
    pub(crate) fn new(header: GenericHeader, infos: Vec<CallInfo>) -> Self {
        Self {
            header,
            infos: infos.into(),
        }
    }
}

impl CallInfoHeader {
    /// Get a reference to the infos from the Call-Info header.
    pub fn infos(&self) -> &CallInfos {
        &self.infos
    }
}

impl HeaderAccessor for CallInfoHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Call-Info")
    }
    fn normalized_value(&self) -> String {
        self.infos.to_string()
    }
}

impl std::fmt::Display for CallInfoHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for CallInfoHeader {
    fn eq(&self, other: &Self) -> bool {
        let self_call_infos: HashSet<_> = self.infos.iter().collect();
        let other_call_infos: HashSet<_> = other.infos.iter().collect();
        self_call_infos == other_call_infos
    }
}

/// Representation of the list of call information from a `Call-Info` header.
///
/// This is usable as an iterator.
pub type CallInfos = HeaderValueCollection<CallInfo>;

impl CallInfos {
    /// Tell whether Call-Info header contains the given `AbsoluteUri`.
    pub fn contains(&self, uri: &AbsoluteUri) -> bool {
        self.iter().any(|info| info.uri == uri)
    }

    /// Get the `CallInfo` corresponding to the given `AbsoluteUri`.
    pub fn get(&self, uri: &AbsoluteUri) -> Option<&CallInfo> {
        self.iter().find(|info| info.uri == uri)
    }
}

/// Representation of a call info, containing its uri and parameters.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
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

impl Hash for CallInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

/// Representation of an information about the caller or the callee.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub enum CallInfoParameter {
    /// The `icon` purpose parameter designates an image suitable as an iconic
    /// representation of the caller or callee.
    IconPurpose,
    /// The `info` purpose parameter describes the caller or callee in general,
    /// for example, through a web page.
    InfoPurpose,
    /// The `card` purpose parameter provides a business card, for example, in
    /// vCard or LDIF formats.
    CardPurpose,
    /// Any other purpose parameter.
    OtherPurpose(String),
    /// Any extension parameter.
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

    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::IconPurpose | Self::InfoPurpose | Self::CardPurpose | Self::OtherPurpose(_) => {
                "purpose"
            }
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
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

impl PartialEq for CallInfoParameter {
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
    fn cmp(&self, other: &Self) -> Ordering {
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
    use crate::{
        header::{
            call_info_header::CallInfoParameter,
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        GenericParameter, Header, Uri,
    };
    use claims::assert_ok;
    use std::str::FromStr;

    valid_header!(CallInfo, CallInfoHeader, "Call-Info");
    header_equality!(CallInfo, "Call-Info");
    header_inequality!(CallInfo, "Call-Info");

    #[test]
    fn test_valid_call_info_header_with_icon_and_info() {
        valid_header("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info", |header| {
            assert_eq!(header.infos().len(), 2);
            let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
            let first_uri = first_uri.as_absolute_uri().unwrap();
            assert!(header.infos().contains(first_uri));
            let first_call_info = header.infos().get(first_uri);
            assert!(first_call_info.is_some());
            let first_call_info = first_call_info.unwrap();
            assert_eq!(first_call_info.parameters().len(), 1);
            assert_eq!(
                first_call_info.parameters().first().unwrap(),
                CallInfoParameter::IconPurpose
            );
            let second_uri = Uri::from_str("http://www.example.com/alice/").unwrap();
            let second_uri = second_uri.as_absolute_uri().unwrap();
            assert!(header.infos().contains(second_uri));
            let second_call_info = header.infos().get(second_uri);
            assert!(second_call_info.is_some());
            let second_call_info = second_call_info.unwrap();
            assert_eq!(second_call_info.parameters().len(), 1);
            assert_eq!(
                second_call_info.parameters().first().unwrap(),
                CallInfoParameter::InfoPurpose
            );
            let third_uri = Uri::from_str("http://www.example.com/bob/").unwrap();
            let third_uri = third_uri.as_absolute_uri().unwrap();
            assert!(!header.infos().contains(third_uri));
        });
    }

    #[test]
    fn test_valid_call_info_header_with_custom_purpose() {
        valid_header(
            "Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=photo",
            |header| {
                assert_eq!(header.infos().len(), 1);
                let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(first_uri));
                let first_call_info = header.infos().get(first_uri);
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
                assert_eq!(header.infos().len(), 1);
                let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(first_uri));
                let first_call_info = header.infos().get(first_uri);
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
                assert_eq!(header.infos().len(), 1);
                let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(first_uri));
                let first_call_info = header.infos().get(first_uri);
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
                assert_eq!(header.infos().len(), 1);
                let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
                let first_uri = first_uri.as_absolute_uri().unwrap();
                assert!(header.infos().contains(first_uri));
                let first_call_info = header.infos().get(first_uri);
                assert!(first_call_info.is_some());
                let first_call_info = first_call_info.unwrap();
                assert!(first_call_info.parameters().is_empty());
            },
        );
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

    #[test]
    fn test_call_info_header_to_string() {
        let header = Header::from_str(
            "call-info:   <http://wwww.example.com/alice/photo.jpg> ;puRpoSe=Icon",
        );
        if let Header::CallInfo(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "call-info:   <http://wwww.example.com/alice/photo.jpg> ;puRpoSe=Icon"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Call-Info: <http://wwww.example.com/alice/photo.jpg>;purpose=icon"
            );
            assert_eq!(
                header.to_compact_string(),
                "Call-Info: <http://wwww.example.com/alice/photo.jpg>;purpose=icon"
            );
        }
    }
}
