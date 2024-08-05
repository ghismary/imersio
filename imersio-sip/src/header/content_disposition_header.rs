//! SIP Content-Disposition header parsing and generation.

use derive_more::Display;
use itertools::join;
use partial_eq_refs::PartialEqRefs;
use std::ops::Deref;

use crate::header::GenericHeader;
use crate::utils::compare_vectors;
use crate::DispositionParameter;
use crate::DispositionType;
use crate::HeaderAccessor;

/// Representation of a Content-Disposition header.
///
/// The Content-Disposition header field describes how the message body or,
/// for multipart messages, a message body part is to be interpreted by the
/// UAC or UAS.
///
/// [[RFC3261, Section 20.11](https://datatracker.ietf.org/doc/html/rfc3261#section-20.11)]
#[derive(Clone, Debug, Display, Eq, PartialEqRefs)]
#[display(fmt = "{}", header)]
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
        Self {
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
            join(&self.parameters, ";")
        )
    }
}

impl PartialEq for ContentDispositionHeader {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type
            && compare_vectors(self.parameters().deref(), other.parameters().deref())
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_ok;

    use super::{ContentDispositionHeader, DispositionType};
    use crate::common::handling::Handling;
    use crate::{
        header::tests::{header_equality, header_inequality, invalid_header, valid_header},
        Header, HeaderAccessor,
    };

    valid_header!(
        ContentDisposition,
        ContentDispositionHeader,
        "Content-Disposition"
    );
    header_equality!(ContentDisposition, "Content-Disposition");
    header_inequality!(ContentDisposition, "Content-Disposition");

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
        let header = Header::try_from("content-disposition:  Session ; HANDLING=OPTIONAL");
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
