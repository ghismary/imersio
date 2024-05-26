use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::GenericParameter;

#[derive(Clone, Debug, Eq)]
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

#[derive(Clone, Debug, Eq)]
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

#[derive(Clone, Debug, Eq)]
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

#[derive(Clone, Debug, Eq)]
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
    use super::{ContentDispositionHeader, DispositionType, Handling};
    use crate::Header;
    use std::str::FromStr;

    fn valid_header<F: FnOnce(ContentDispositionHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert!(header.is_ok());
        if let Header::ContentDisposition(header) = header.unwrap() {
            f(header);
        } else {
            panic!("Not a Content-Disposition header");
        }
    }

    #[test]
    fn test_valid_content_disposition_header() {
        valid_header("Content-Disposition: session", |header| {
            assert_eq!(header.r#type(), DispositionType::Session);
            assert!(header.parameters().is_empty());
        });
    }

    #[test]
    fn test_valid_content_disposition_header_with_parameter() {
        valid_header("Content-Disposition: session;handling=optional", |header| {
            assert_eq!(header.r#type(), DispositionType::Session);
            assert_eq!(header.parameters().len(), 1);
            assert_eq!(
                header.parameters().first().unwrap().handling(),
                Some(&Handling::Optional)
            )
        });
    }

    #[test]
    fn test_valid_content_disposition_header_with_custom_type() {
        valid_header("Content-Disposition: custom", |header| {
            assert_eq!(
                header.r#type(),
                DispositionType::Other("custom".to_string())
            );
            assert!(header.parameters().is_empty());
        });
    }

    fn invalid_header(header: &str) {
        assert!(Header::from_str(header).is_err());
    }

    #[test]
    fn test_invalid_content_disposition_header_empty() {
        invalid_header("Content-Disposition:");
    }

    #[test]
    fn test_invalid_content_disposition_header_empty_with_space_characters() {
        invalid_header("Content-Disposition:    ");
    }

    #[test]
    fn test_invalid_content_disposition_header_with_invalid_character() {
        invalid_header("Content-Disposition: 😁");
    }

    fn header_equality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (
            Header::ContentDisposition(first_header),
            Header::ContentDisposition(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_eq!(first_header, second_header);
        } else {
            panic!("Not a Content-Disposition header");
        }
    }

    #[test]
    fn test_content_disposition_header_equality_with_space_characters_differences() {
        header_equality(
            "Content-Disposition: session",
            "Content-Disposition:   session",
        );
    }

    #[test]
    fn test_content_disposition_header_equality_parameters_in_a_different_order() {
        header_equality(
            "Content-Disposition: session;handling=required;myparam=test",
            "Content-Disposition: session;myparam=test;handling=required",
        );
    }

    #[test]
    fn test_content_disposition_header_equality_with_different_cases() {
        header_equality(
            "Content-Disposition: session;handling=optional",
            "content-disposition: Session;HANDLING=OPTIONAL",
        );
    }

    fn header_inequality(first_header: &str, second_header: &str) {
        let first_header = Header::from_str(first_header);
        let second_header = Header::from_str(second_header);
        if let (
            Header::ContentDisposition(first_header),
            Header::ContentDisposition(second_header),
        ) = (first_header.unwrap(), second_header.unwrap())
        {
            assert_ne!(first_header, second_header);
        } else {
            panic!("Not a Content-Disposition header");
        }
    }

    #[test]
    fn test_content_disposition_header_inequality_with_different_types() {
        header_inequality(
            "Content-Disposition: session",
            "Content-Disposition: render",
        );
    }

    #[test]
    fn test_content_disposition_header_inequality_with_same_type_but_one_has_a_parameter() {
        header_inequality(
            "Content-Disposition: session",
            "Content-Disposition: session;handling=required",
        );
    }

    #[test]
    fn test_content_disposition_header_inequality_with_same_parameter_but_different_types() {
        header_inequality(
            "Content-Disposition: session;handling=optional",
            "Content-Disposition: render;handling=optional",
        );
    }
}
