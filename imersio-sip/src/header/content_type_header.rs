//! SIP Content-Type header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::header::GenericHeader;
use crate::HeaderAccessor;
use crate::MediaType;

/// Representation of a Content-Type header.
///
/// The Content-Type header field indicates the media type of the message body
/// sent to the recipient. The Content-Type header field MUST be present if
/// the body is not empty. If the body is empty, and a Content-Type header
/// field is present, it indicates that the body of the specific type has
/// zero length (for example, an empty audio file).
///
/// [[RFC3261, Section 20.15](https://datatracker.ietf.org/doc/html/rfc3261#section-20.15)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct ContentTypeHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    media_type: MediaType,
}

impl ContentTypeHeader {
    pub(crate) fn new(header: GenericHeader, media_type: MediaType) -> Self {
        Self { header, media_type }
    }

    /// Get a reference to the media type from the `Content` header.
    pub fn media_type(&self) -> &MediaType {
        &self.media_type
    }
}

impl HeaderAccessor for ContentTypeHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("c")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Content-Type")
    }
    fn normalized_value(&self) -> String {
        self.media_type.to_string()
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_ok;

    use super::ContentTypeHeader;
    use crate::common::media_parameter::MediaParameter;
    use crate::{
        common::{media_range::MediaRange, wrapped_string::WrappedString},
        header::{
            tests::header_equality, tests::header_inequality, tests::invalid_header,
            tests::valid_header,
        },
        Header, HeaderAccessor,
    };

    valid_header!(ContentType, ContentTypeHeader, "Content-Type");
    header_equality!(ContentType, "Content-Type");
    header_inequality!(ContentType, "Content-Type");

    #[test]
    fn test_valid_content_type_header_without_parameters() {
        valid_header("Content-Type: application/sdp", |header| {
            assert_eq!(
                header.media_type().media_range(),
                &MediaRange::new("application", "sdp")
            );
        });
    }

    #[test]
    fn test_valid_content_type_header_with_parameters() {
        valid_header("c: text/html; charset=ISO-8859-4", |header| {
            let media_type = header.media_type();
            assert_eq!(media_type.media_range(), &MediaRange::new("text", "html"));
            assert_eq!(media_type.parameters().len(), 1);
            assert_eq!(
                media_type.parameters().first().unwrap(),
                MediaParameter::new("charset", WrappedString::new_not_wrapped("ISO-8859-4"))
            );
        });
    }

    #[test]
    fn test_invalid_content_type_header_empty() {
        invalid_header("Content-Type:");
    }

    #[test]
    fn test_invalid_content_type_header_empty_with_space_characters() {
        invalid_header("Content-Type:      ");
    }

    #[test]
    fn test_invalid_content_type_header_only_range_type() {
        invalid_header("Content-Type: application");
    }

    #[test]
    fn test_invalid_content_type_header_only_range_type_and_slash() {
        invalid_header("Content-Type: application/");
    }

    #[test]
    fn test_invalid_content_type_header_invalid_characters() {
        invalid_header("Content-Type: 😁/😁");
    }

    #[test]
    fn test_content_type_header_equality_same_headers_with_just_space_characters_differences() {
        header_equality("Content-Type: text/html", "Content-Type:  text/html");
    }

    #[test]
    fn test_content_type_header_equality_same_headers_one_normal_form_the_other_in_compact_form() {
        header_equality("Content-Type: application/sdp", "c: application/sdp");
    }

    #[test]
    fn test_content_type_header_inequality_with_different_media_types() {
        header_inequality("Content-Type: application/sdp", "Content-Type: text/html");
    }

    #[test]
    fn test_content_type_header_inequality_same_media_types_but_one_with_parameters() {
        header_inequality(
            "Content-Type: text/html; charset=ISO-8859-4",
            "Content-Type: text/html",
        );
    }

    #[test]
    fn test_content_type_header_inequality_same_media_types_but_different_parameters() {
        header_inequality(
            "Content-Type: text/html; charset=ISO-8859-4",
            "Content-Type: text/html; charset=ISO-8859-15",
        );
    }

    #[test]
    fn test_content_type_header_to_string() {
        let header = Header::try_from("content-typE  :  text/html ; charset=  ISO-8859-4");
        if let Header::ContentType(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                "content-typE  :  text/html ; charset=  ISO-8859-4"
            );
            assert_eq!(
                header.to_normalized_string(),
                "Content-Type: text/html;charset=ISO-8859-4"
            );
            assert_eq!(
                header.to_compact_string(),
                "c: text/html;charset=ISO-8859-4"
            );
        }
    }
}
