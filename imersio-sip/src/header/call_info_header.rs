use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::{uri::AbsoluteUri, GenericParameter};

#[derive(Clone, Debug)]
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

impl Eq for CallInfoHeader {}

#[derive(Clone, Debug)]
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

impl Eq for CallInfo {}

impl Hash for CallInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

#[derive(Clone, Debug)]
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

impl Eq for CallInfoParameter {}

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
    use crate::{header::call_info_header::CallInfoParameter, GenericParameter, Header, Uri};
    use std::str::FromStr;

    #[test]
    fn test_valid_call_info_header() {
        // Valid Call-Info header with icon and info.
        let header = Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info");
        assert!(header.is_ok());
        if let Header::CallInfo(header) = header.unwrap() {
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
        } else {
            panic!("Not an Call-Info header");
        }

        // Valid Call-Info header with custom purpose.
        let header =
            Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=photo");
        assert!(header.is_ok());
        if let Header::CallInfo(header) = header.unwrap() {
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
        } else {
            panic!("Not an Call-Info header");
        }

        // Valid Call-Info header with custom param with value.
        let header =
            Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;info=photo");
        assert!(header.is_ok());
        if let Header::CallInfo(header) = header.unwrap() {
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
        } else {
            panic!("Not an Call-Info header");
        }

        // Valid Call-Info header with custom param without value.
        let header = Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;info");
        assert!(header.is_ok());
        if let Header::CallInfo(header) = header.unwrap() {
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
        } else {
            panic!("Not an Call-Info header");
        }

        // Valid Call-Info header without param.
        let header = Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg>");
        assert!(header.is_ok());
        if let Header::CallInfo(header) = header.unwrap() {
            assert_eq!(header.count(), 1);
            let first_uri = Uri::from_str("http://wwww.example.com/alice/photo.jpg").unwrap();
            let first_uri = first_uri.as_absolute_uri().unwrap();
            assert!(header.contains(first_uri));
            let first_call_info = header.get(first_uri);
            assert!(first_call_info.is_some());
            let first_call_info = first_call_info.unwrap();
            assert!(first_call_info.parameters().is_empty());
        } else {
            panic!("Not an Call-Info header");
        }
    }

    #[test]
    fn test_invalid_call_info_header() {
        // Empty Call-Info header.
        let header = Header::from_str("Call-Info:");
        assert!(header.is_err());

        // Empty Call-Info header with spaces.
        let header = Header::from_str("Call-Info:    ");
        assert!(header.is_err());

        // Call-Info header with invalid character.
        let header = Header::from_str("Call-Info: 😁");
        assert!(header.is_err());

        // Call-Info header with invalid uri.
        let header = Header::from_str("Call-Info: http://wwww.example.com/alice/photo.jpg");
        assert!(header.is_err());
    }

    #[test]
    fn test_call_info_header_equality() {
        // Same Call-Info header.
        let first_header = Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info");
        let second_header = Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info");
        if let (Header::CallInfo(first_header), Header::CallInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Call-Info header");
        }

        // Same Call-Info header with inverted infos.
        let first_header = Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info");
        let second_header = Header::from_str("Call-Info: <http://www.example.com/alice/> ;purpose=info, <http://wwww.example.com/alice/photo.jpg> ;purpose=icon");
        if let (Header::CallInfo(first_header), Header::CallInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Call-Info header");
        }

        // Same Call-Info headers with different cases.
        let first_header =
            Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon");
        let second_header =
            Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;puRpoSe=Icon");
        if let (Header::CallInfo(first_header), Header::CallInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Call-Info header");
        }
    }

    #[test]
    fn test_call_info_header_inequality() {
        // Different uris with same purpose.
        let first_header =
            Header::from_str("Call-Info: <http://www.example.com/alice/> ;purpose=info");
        let second_header =
            Header::from_str("Call-Info: <http://www.example.com/bob/> ;purpose=info");
        if let (Header::CallInfo(first_header), Header::CallInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Call-Info header");
        }

        // Same uris with different purpose.
        let first_header =
            Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=icon");
        let second_header =
            Header::from_str("Call-Info: <http://wwww.example.com/alice/photo.jpg> ;purpose=info");
        if let (Header::CallInfo(first_header), Header::CallInfo(second_header)) =
            (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Call-Info header");
        }
    }
}
