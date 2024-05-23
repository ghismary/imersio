use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::GenericParameter;

#[derive(Clone, Debug)]
pub struct ContentDispositionHeader {
    r#type: DispositionType,
    parameters: Vec<DispositionParameter>,
}

impl ContentDispositionHeader {
    pub(crate) fn new(r#type: DispositionType, parameters: Vec<DispositionParameter>) -> Self {
        ContentDispositionHeader { r#type, parameters }
    }

    /// Get a reference to the type from the ContentDisposition header.
    pub fn r#type(&self) -> &DispositionType {
        &self.r#type
    }

    /// Get a reference to the parameters from the ContentDisposition header.
    pub fn parameters(&self) -> &Vec<DispositionParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for ContentDispositionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Content-Disposition: {}{}{}",
            self.r#type,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl PartialEq for ContentDispositionHeader {
    fn eq(&self, other: &Self) -> bool {
        if self.r#type != other.r#type {
            return false;
        }
        let self_params: HashSet<_> = self.parameters().iter().collect();
        let other_params: HashSet<_> = other.parameters().iter().collect();
        self_params == other_params
    }
}

impl PartialEq<&ContentDispositionHeader> for ContentDispositionHeader {
    fn eq(&self, other: &&ContentDispositionHeader) -> bool {
        self == *other
    }
}

impl PartialEq<ContentDispositionHeader> for &ContentDispositionHeader {
    fn eq(&self, other: &ContentDispositionHeader) -> bool {
        *self == other
    }
}

impl Eq for ContentDispositionHeader {}

#[derive(Clone, Debug)]
pub enum DispositionType {
    Render,
    Session,
    Icon,
    Alert,
    Other(String),
}

impl DispositionType {
    pub(crate) fn new<S: Into<String>>(r#type: S) -> DispositionType {
        let r#type: String = r#type.into();
        match r#type.to_ascii_lowercase().as_ref() {
            "render" => Self::Render,
            "session" => Self::Session,
            "icon" => Self::Icon,
            "alert" => Self::Alert,
            _ => Self::Other(r#type),
        }
    }
}

impl std::fmt::Display for DispositionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Render => "render",
                Self::Session => "session",
                Self::Icon => "icon",
                Self::Alert => "alert",
                Self::Other(value) => value,
            }
        )
    }
}

impl PartialEq<DispositionType> for DispositionType {
    fn eq(&self, other: &DispositionType) -> bool {
        match (self, other) {
            (Self::Render, Self::Render)
            | (Self::Session, Self::Session)
            | (Self::Icon, Self::Icon)
            | (Self::Alert, Self::Alert) => true,
            (Self::Other(svalue), Self::Other(ovalue)) => svalue.eq_ignore_ascii_case(ovalue),
            _ => false,
        }
    }
}

impl PartialEq<&DispositionType> for DispositionType {
    fn eq(&self, other: &&DispositionType) -> bool {
        self == *other
    }
}

impl PartialEq<DispositionType> for &DispositionType {
    fn eq(&self, other: &DispositionType) -> bool {
        *self == other
    }
}

impl Eq for DispositionType {}

#[derive(Clone, Debug)]
pub enum DispositionParameter {
    Handling(Handling),
    Other(GenericParameter),
}

impl DispositionParameter {
    pub fn key(&self) -> &str {
        match self {
            Self::Handling(_) => "handling",
            Self::Other(param) => param.key(),
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Handling(value) => Some(value.value()),
            Self::Other(param) => param.value(),
        }
    }

    pub fn handling(&self) -> Option<&Handling> {
        match self {
            Self::Handling(value) => Some(value),
            _ => None,
        }
    }
}

impl std::fmt::Display for DispositionParameter {
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

impl PartialEq<DispositionParameter> for DispositionParameter {
    fn eq(&self, other: &DispositionParameter) -> bool {
        match (self, other) {
            (Self::Handling(shandling), Self::Handling(ohandling)) => shandling == ohandling,
            (Self::Other(sparam), Self::Other(oparam)) => sparam == oparam,
            _ => false,
        }
    }
}

impl PartialEq<&DispositionParameter> for DispositionParameter {
    fn eq(&self, other: &&DispositionParameter) -> bool {
        self == *other
    }
}

impl PartialEq<DispositionParameter> for &DispositionParameter {
    fn eq(&self, other: &DispositionParameter) -> bool {
        *self == other
    }
}

impl PartialOrd for DispositionParameter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for DispositionParameter {}

impl Hash for DispositionParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state);
        self.value().hash(state);
    }
}

impl Ord for DispositionParameter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.key().cmp(other.key()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(&other.value())
    }
}

impl From<GenericParameter> for DispositionParameter {
    fn from(value: GenericParameter) -> Self {
        Self::Other(value)
    }
}

#[derive(Clone, Debug)]
pub enum Handling {
    Optional,
    Required,
    Other(String),
}

impl Handling {
    pub(crate) fn new<S: Into<String>>(handling: S) -> Handling {
        let handling: String = handling.into();
        match handling.to_ascii_lowercase().as_str() {
            "optional" => Self::Optional,
            "required" => Self::Required,
            _ => Self::Other(handling),
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Optional => "optional",
            Self::Required => "required",
            Self::Other(value) => value,
        }
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Optional)
    }

    pub fn is_required(&self) -> bool {
        matches!(self, Self::Required)
    }
}

impl std::fmt::Display for Handling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq<Handling> for Handling {
    fn eq(&self, other: &Handling) -> bool {
        match (self, other) {
            (Self::Optional, Self::Optional) | (Self::Required, Self::Required) => true,
            (Self::Other(svalue), Self::Other(ovalue)) => svalue.eq_ignore_ascii_case(ovalue),
            _ => false,
        }
    }
}

impl PartialEq<&Handling> for Handling {
    fn eq(&self, other: &&Handling) -> bool {
        self == *other
    }
}

impl PartialEq<Handling> for &Handling {
    fn eq(&self, other: &Handling) -> bool {
        *self == other
    }
}

impl Eq for Handling {}

impl Hash for Handling {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}

impl PartialOrd for Handling {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Handling {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(other.value())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        header::content_disposition_header::{DispositionType, Handling},
        Header,
    };
    use std::str::FromStr;

    #[test]
    fn test_valid_content_disposition_header() {
        // Valid Content-Disposition header.
        let header = Header::from_str("Content-Disposition: session");
        assert!(header.is_ok());
        if let Header::ContentDisposition(header) = header.unwrap() {
            assert_eq!(header.r#type(), DispositionType::Session);
            assert!(header.parameters().is_empty());
        } else {
            panic!("Not an Content-Disposition header");
        }

        // Valid Content-Disposition header with parameter.
        let header = Header::from_str("Content-Disposition: session;handling=optional");
        assert!(header.is_ok());
        if let Header::ContentDisposition(header) = header.unwrap() {
            assert_eq!(header.r#type(), DispositionType::Session);
            assert_eq!(header.parameters().len(), 1);
            assert_eq!(
                header.parameters().first().unwrap().handling(),
                Some(&Handling::Optional)
            )
        } else {
            panic!("Not an Content-Disposition header");
        }

        // Valid Content-Disposition header with custom type.
        let header = Header::from_str("Content-Disposition: custom");
        assert!(header.is_ok());
        if let Header::ContentDisposition(header) = header.unwrap() {
            assert_eq!(
                header.r#type(),
                DispositionType::Other("custom".to_string())
            );
            assert!(header.parameters().is_empty());
        } else {
            panic!("Not an Content-Disposition header");
        }
    }

    #[test]
    fn test_invalid_content_disposition_header() {
        // Empty Content-Disposition header.
        let header = Header::from_str("Content-Disposition:");
        assert!(header.is_err());

        // Empty Content-Disposition header with spaces.
        let header = Header::from_str("Content-Disposition:    ");
        assert!(header.is_err());

        // Content-Disposition header with invalid character.
        let header = Header::from_str("Content-Disposition: 😁");
        assert!(header.is_err());
    }

    #[test]
    fn test_content_disposition_header_equality() {
        // Same Content-Disposition headers, just some minor spaces differences.
        let first_header = Header::from_str("Content-Disposition: session");
        let second_header = Header::from_str("Content-Disposition:   session");
        if let (
            Header::ContentDisposition(first_header),
            Header::ContentDisposition(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Content-Disposition header");
        }

        // Same Content-Disposition headers, with parameters in different orders.
        let first_header =
            Header::from_str("Content-Disposition: session;handling=required;myparam=test");
        let second_header =
            Header::from_str("Content-Disposition: session;myparam=test;handling=required");
        if let (
            Header::ContentDisposition(first_header),
            Header::ContentDisposition(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Content-Disposition header");
        }

        // Same Content-Disposition headers, but with different cases.
        let first_header = Header::from_str("Content-Disposition: session;handling=optional");
        let second_header = Header::from_str("content-disposition: Session;HANDLING=OPTIONAL");
        if let (
            Header::ContentDisposition(first_header),
            Header::ContentDisposition(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not an Content-Disposition header");
        }
    }

    #[test]
    fn test_content_disposition_header_inequality() {
        // Different disposition types.
        let first_header = Header::from_str("Content-Disposition: session");
        let second_header = Header::from_str("Content-Disposition: render");
        if let (
            Header::ContentDisposition(first_header),
            Header::ContentDisposition(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Content-Disposition header");
        }

        // Same disposition type, but one has a parameter.
        let first_header = Header::from_str("Content-Disposition: session");
        let second_header = Header::from_str("Content-Disposition: session;handling=required");
        if let (
            Header::ContentDisposition(first_header),
            Header::ContentDisposition(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Content-Disposition header");
        }

        // Same parameter but different disposition type.
        let first_header = Header::from_str("Content-Disposition: session;handling=optional");
        let second_header = Header::from_str("Content-Disposition: render;handling=optional");
        if let (
            Header::ContentDisposition(first_header),
            Header::ContentDisposition(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not an Content-Disposition header");
        }
    }
}
