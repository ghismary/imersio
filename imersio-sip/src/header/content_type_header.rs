use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use partial_eq_refs::PartialEqRefs;

use crate::{
    common::{media_range::MediaRange, wrapped_string::WrappedString},
    HeaderAccessor,
};

use super::generic_header::GenericHeader;

/// Representation of a Content-Type header.
///
/// The Content-Type header field indicates the media type of the message body
/// sent to the recipient. The Content-Type header field MUST be present if
/// the body is not empty. If the body is empty, and a Content-Type header
/// field is present, it indicates that the body of the specific type has
/// zero length (for example, an empty audio file).
///
/// [[RFC3261, Section 20.15](https://datatracker.ietf.org/doc/html/rfc3261#section-20.15)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct ContentTypeHeader {
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

impl std::fmt::Display for ContentTypeHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for ContentTypeHeader {
    fn eq(&self, other: &Self) -> bool {
        self.media_type == other.media_type
    }
}

/// Representation of a media type contained in a `ContentTypeHeader`.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct MediaType {
    media_range: MediaRange,
    parameters: Vec<MediaParameter>,
}

impl MediaType {
    pub(crate) fn new(media_range: MediaRange, parameters: Vec<MediaParameter>) -> Self {
        MediaType {
            media_range,
            parameters,
        }
    }

    /// Get a reference to the `MediaRange` of the media type.
    pub fn media_range(&self) -> &MediaRange {
        &self.media_range
    }

    /// Get a reference to the list of `MediaParameter`s of the media type.
    pub fn parameters(&self) -> &Vec<MediaParameter> {
        &self.parameters
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.media_range,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl PartialEq for MediaType {
    fn eq(&self, other: &Self) -> bool {
        if self.media_range != other.media_range {
            return false;
        }

        let self_params: HashSet<_> = self.parameters.iter().collect();
        let other_params: HashSet<_> = other.parameters.iter().collect();
        self_params == other_params
    }
}

impl Hash for MediaType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.media_range.hash(state);
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort();
        sorted_params.hash(state);
    }
}

/// Representation of a media parameter.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct MediaParameter {
    key: String,
    value: WrappedString,
}

impl MediaParameter {
    /// Create a `MediaParameter`.
    pub fn new<S: Into<String>>(key: S, value: WrappedString) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }

    /// Get the key of the media parameter.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the value of the media parameter.
    pub fn value(&self) -> &WrappedString {
        &self.value
    }
}

impl std::fmt::Display for MediaParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

impl PartialEq for MediaParameter {
    fn eq(&self, other: &MediaParameter) -> bool {
        self.key().eq_ignore_ascii_case(other.key()) && self.value() == other.value()
    }
}

impl PartialOrd for MediaParameter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MediaParameter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self
            .key()
            .to_ascii_lowercase()
            .cmp(&other.key().to_ascii_lowercase())
        {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.value().cmp(other.value())
    }
}

impl Hash for MediaParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().to_ascii_lowercase().hash(state);
        self.value().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::ContentTypeHeader;
    use crate::{
        common::{media_range::MediaRange, wrapped_string::WrappedString},
        header::{
            tests::header_equality, tests::header_inequality, tests::invalid_header,
            tests::valid_header, MediaParameter,
        },
        Header, HeaderAccessor,
    };
    use claim::assert_ok;
    use std::str::FromStr;

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
        let header = Header::from_str("content-typE  :  text/html ; charset=  ISO-8859-4");
        if let Header::Accept(header) = header.unwrap() {
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
