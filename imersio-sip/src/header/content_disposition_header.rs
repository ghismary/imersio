use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::{utils::partial_eq_refs, GenericParameter};

use super::{generic_header::GenericHeader, HeaderAccessor};

/// Representation of a Content-Disposition header.
///
/// The Content-Disposition header field describes how the message body or,
/// for multipart messages, a message body part is to be interpreted by the
/// UAC or UAS.
///
/// [[RFC3261, Section 20.11](https://datatracker.ietf.org/doc/html/rfc3261#section-20.11)]
#[derive(Clone, Debug, Eq)]
pub struct ContentDispositionHeader {
    header: GenericHeader,
    r#type: DispositionType,
    parameters: Vec<DispositionParameter>,
}

impl ContentDispositionHeader {
    pub(crate) fn new(
        header: GenericHeader,
        r#type: DispositionType,
        parameters: Vec<DispositionParameter>,
    ) -> Self {
        ContentDispositionHeader {
            header,
            r#type,
            parameters,
        }
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

impl HeaderAccessor for ContentDispositionHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Content-Disposition")
    }
    fn normalized_value(&self) -> String {
        format!(
            "{}{}{}",
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

impl std::fmt::Display for ContentDispositionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
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

partial_eq_refs!(ContentDispositionHeader);

/// Representation of a disposition type from a `Content-Disposition` header.
#[derive(Clone, Debug, Eq)]
pub enum DispositionType {
    /// The value `render` indicates that the body part should be displayed or
    /// otherwise rendered to the user.
    Render,
    /// The value `session` indicates that the body part describes a session,
    /// for either calls or early (pre-call) media.
    Session,
    /// The value `icon` indicates that the body part contains an image
    /// suitable as an iconic representation of the caller or callee that
    /// could be rendered informationally by a user agent when a message has
    /// been received, or persistently while a dialog takes place.
    Icon,
    /// The value `alert`` indicates that the body part contains information,
    /// such as an audio clip, that should be rendered by the user agent in an
    /// attempt to alert the user to the receipt of a request.
    Alert,
    /// Any other extension disposition type.
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

partial_eq_refs!(DispositionType);

/// Representation of a parameter of a `DispositionType`.
#[derive(Clone, Debug, Eq)]
pub enum DispositionParameter {
    /// The handling parameter describes how the UAS should react if it
    /// receives a message body whose content type or disposition type it
    /// does not understand.
    Handling(HandlingValue),
    /// Any other parameter.
    Other(GenericParameter),
}

impl DispositionParameter {
    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Handling(_) => "handling",
            Self::Other(param) => param.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Handling(value) => Some(value.value()),
            Self::Other(param) => param.value(),
        }
    }

    /// Get the handling value of the parameter if this is a `handling`
    /// parameter.
    pub fn handling(&self) -> Option<&HandlingValue> {
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

partial_eq_refs!(DispositionParameter);

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

/// Representation of the `handling` parameter of a `DispositionType`.
#[derive(Clone, Debug, Eq)]
pub enum HandlingValue {
    /// The handling of the content type is optional.
    Optional,
    /// The handling of the content type is required.
    Required,
    /// Any extension value.
    Other(String),
}

impl HandlingValue {
    pub(crate) fn new<S: Into<String>>(handling: S) -> HandlingValue {
        let handling: String = handling.into();
        match handling.to_ascii_lowercase().as_str() {
            "optional" => Self::Optional,
            "required" => Self::Required,
            _ => Self::Other(handling),
        }
    }

    /// Get the value of the `HandlingValue.`
    pub fn value(&self) -> &str {
        match self {
            Self::Optional => "optional",
            Self::Required => "required",
            Self::Other(value) => value,
        }
    }

    /// Tell whether the parameter has the `optional` value.
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Optional)
    }

    /// Tell whether the parameter has the `required` value.
    pub fn is_required(&self) -> bool {
        matches!(self, Self::Required)
    }
}

impl std::fmt::Display for HandlingValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq<HandlingValue> for HandlingValue {
    fn eq(&self, other: &HandlingValue) -> bool {
        match (self, other) {
            (Self::Optional, Self::Optional) | (Self::Required, Self::Required) => true,
            (Self::Other(svalue), Self::Other(ovalue)) => svalue.eq_ignore_ascii_case(ovalue),
            _ => false,
        }
    }
}

partial_eq_refs!(HandlingValue);

impl Hash for HandlingValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}

impl PartialOrd for HandlingValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HandlingValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(other.value())
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentDispositionHeader, DispositionType, HandlingValue};
    use crate::{Header, HeaderAccessor};
    use claim::{assert_err, assert_ok};
    use std::str::FromStr;

    fn valid_header<F: FnOnce(ContentDispositionHeader)>(header: &str, f: F) {
        let header = Header::from_str(header);
        assert_ok!(&header);
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
                Some(&HandlingValue::Optional)
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
        assert_err!(Header::from_str(header));
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

    #[test]
    fn test_content_disposition_header_to_string() {
        let header = Header::from_str("content-disposition:  Session ; HANDLING=OPTIONAL");
        if let Header::ContentDisposition(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "content-disposition:  Session ; HANDLING=OPTIONAL"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Content-Disposition: session;handling=optional"
            );
            assert_eq!(
                header.to_compact_string(),
                "Content-Disposition: session;handling=optional"
            );
        }
    }
}
