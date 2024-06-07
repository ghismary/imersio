use std::{cmp::Ordering, collections::HashSet, hash::Hash, ops::Deref};

use partial_eq_refs::PartialEqRefs;

use crate::{common::name_address::NameAddress, GenericParameter, HeaderAccessor};

use super::generic_header::GenericHeader;

/// Representation of a From header.
///
/// The From header field indicates the initiator of the request. This may be different from the
/// initiator of the dialog. Requests sent by the callee to the caller use the callee's address in
/// the From header field.
///
/// [[RFC3261, Section 20.20](https://datatracker.ietf.org/doc/html/rfc3261#section-20.20)]
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct FromHeader {
    header: GenericHeader,
    address: NameAddress,
    parameters: FromParameters,
}

impl FromHeader {
    pub(crate) fn new(
        header: GenericHeader,
        address: NameAddress,
        parameters: Vec<FromParameter>,
    ) -> Self {
        Self {
            header,
            address,
            parameters: parameters.into(),
        }
    }

    /// Get a reference to the address from the From header.
    pub fn address(&self) -> &NameAddress {
        &self.address
    }

    /// Get a reference to the parameters from the From header.
    pub fn parameters(&self) -> &FromParameters {
        &self.parameters
    }

    /// Get the value of the `tag` parameter from the From header, if it has one.
    pub fn tag(&self) -> Option<&str> {
        self.parameters
            .iter()
            .find(|param| matches!(param, FromParameter::Tag(_)))
            .and_then(|param| param.tag())
    }
}

impl HeaderAccessor for FromHeader {
    crate::header::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        Some("f")
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("From")
    }
    fn normalized_value(&self) -> String {
        format!(
            "{}{}{}",
            self.address,
            if self.parameters.is_empty() { "" } else { ";" },
            self.parameters
        )
    }
}

impl std::fmt::Display for FromHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
    }
}

impl PartialEq for FromHeader {
    fn eq(&self, other: &FromHeader) -> bool {
        self.address == other.address && self.parameters == other.parameters
    }
}

/// Representation of the list of from parameters of a `From` header.
///
/// This is usable as an iterator.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub struct FromParameters(Vec<FromParameter>);

impl From<Vec<FromParameter>> for FromParameters {
    fn from(value: Vec<FromParameter>) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for FromParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl PartialEq for FromParameters {
    fn eq(&self, other: &Self) -> bool {
        let self_parameters: HashSet<_> = self.0.iter().collect();
        let other_parameters: HashSet<_> = other.0.iter().collect();
        self_parameters == other_parameters
    }
}

impl IntoIterator for FromParameters {
    type Item = FromParameter;
    type IntoIter = <Vec<FromParameter> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for FromParameters {
    type Target = [FromParameter];

    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}

/// Representation of a parameter founded in a `From` header.
#[derive(Clone, Debug, Eq, PartialEqRefs)]
pub enum FromParameter {
    /// A `tag` parameter.
    Tag(String),
    /// Any other parameters.
    Other(GenericParameter),
}

impl FromParameter {
    /// Get the `tag` value if the parameter is a `tag` parameter.
    pub fn tag(&self) -> Option<&str> {
        match self {
            Self::Tag(value) => Some(value),
            _ => None,
        }
    }

    /// Get the key of the parameter.
    pub fn key(&self) -> &str {
        match self {
            Self::Tag(_) => "tag",
            Self::Other(value) => value.key(),
        }
    }

    /// Get the value of the parameter.
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Tag(value) => Some(value),
            Self::Other(value) => value.value(),
        }
    }
}

impl std::fmt::Display for FromParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tag(value) => write!(f, "tag={value}"),
            Self::Other(value) => write!(
                f,
                "{}{}{}",
                value.key().to_ascii_lowercase(),
                if value.value().is_some() { "=" } else { "" },
                value.value().unwrap_or_default().to_ascii_lowercase()
            ),
        }
    }
}

impl PartialEq for FromParameter {
    fn eq(&self, other: &FromParameter) -> bool {
        match (self, other) {
            (Self::Tag(a), Self::Tag(b)) => a == b,
            (Self::Other(a), Self::Other(b)) => {
                a.key().eq_ignore_ascii_case(b.key())
                    && a.value().map(|v| v.to_ascii_lowercase())
                        == b.value().map(|v| v.to_ascii_lowercase())
            }
            _ => false,
        }
    }
}

impl Hash for FromParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Tag(value) => {
                "tag".hash(state);
                value.hash(state);
            }
            Self::Other(param) => param.hash(state),
        }
    }
}

impl PartialOrd for FromParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FromParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .key()
            .to_ascii_lowercase()
            .cmp(&other.key().to_ascii_lowercase())
        {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self {
            Self::Tag(value) => Some(value.as_str()).cmp(&other.value()),
            Self::Other(param) => param
                .value()
                .map(|value| value.to_ascii_lowercase())
                .cmp(&other.value().map(|value| value.to_ascii_lowercase())),
        }
    }
}

impl From<GenericParameter> for FromParameter {
    fn from(value: GenericParameter) -> Self {
        match value.key().to_ascii_lowercase().as_str() {
            "tag" => Self::Tag(value.value().unwrap_or("").to_string()),
            _ => Self::Other(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FromHeader;
    use crate::{
        header::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, Uri,
    };
    use claims::assert_ok;
    use std::str::FromStr;

    valid_header!(From, FromHeader, "From");
    header_equality!(From, "From");
    header_inequality!(From, "From");

    #[test]
    fn test_valid_from_header_with_display_name() {
        valid_header(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            |header| {
                assert_eq!(header.address().display_name(), Some("A. G. Bell"));
                assert_eq!(
                    header.address().uri(),
                    Uri::from_str("sip:agb@bell-telephone.com").unwrap()
                );
                assert_eq!(header.parameters().len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("a48s"));
                assert_eq!(header.tag(), Some("a48s"));
            },
        );
    }

    #[test]
    fn test_valid_from_header_without_display_name() {
        valid_header(
            "From: <sip:+12125551212@server.phone2net.com>;tag=887s",
            |header| {
                assert_eq!(header.address().display_name(), None);
                assert_eq!(
                    header.address().uri(),
                    Uri::from_str("sip:+12125551212@server.phone2net.com").unwrap()
                );
                assert_eq!(header.parameters.len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("887s"));
                assert_eq!(header.tag(), Some("887s"));
            },
        )
    }

    #[test]
    fn test_valid_from_header_in_compact_form() {
        valid_header(
            "f: Anonymous <sip:c8oqz84zk7z@privacy.org>;tag=hyh8",
            |header| {
                assert_eq!(header.address().display_name(), Some("Anonymous"));
                assert_eq!(
                    header.address().uri(),
                    Uri::from_str("sip:c8oqz84zk7z@privacy.org").unwrap()
                );
                assert_eq!(header.parameters.len(), 1);
                let first_parameter = header.parameters().first().unwrap();
                assert_eq!(first_parameter.key(), "tag");
                assert_eq!(first_parameter.value(), Some("hyh8"));
                assert_eq!(header.tag(), Some("hyh8"));
            },
        )
    }

    #[test]
    fn test_invalid_from_header_empty() {
        invalid_header("From:");
    }

    #[test]
    fn test_invalid_from_header_empty_with_space_characters() {
        invalid_header("From:    ");
    }

    #[test]
    fn test_invalid_from_header_with_invalid_character() {
        invalid_header("From: 😁");
    }

    #[test]
    fn test_from_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From:    "A. G. Bell"  <sip:agb@bell-telephone.com>; tag=a48s"#,
        );
    }

    #[test]
    fn test_from_header_equality_same_header_with_different_display_names() {
        header_equality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: Bell <sip:agb@bell-telephone.com> ;tag=a48s"#,
        );
    }

    #[test]
    fn test_from_header_equality_same_header_with_different_cases() {
        header_equality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;TAG=a48s"#,
        );
    }

    #[test]
    fn test_from_header_inequality_different_uris() {
        header_inequality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: "A. G. Bell" <sip:agc@bell-telephone.com> ;tag=a48s"#,
        );
    }

    #[test]
    fn test_from_header_inequality_different_tag_parameters() {
        header_inequality(
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=a48s"#,
            r#"From: "A. G. Bell" <sip:agb@bell-telephone.com> ;tag=hyh8"#,
        );
    }

    #[test]
    fn test_from_header_to_string() {
        let header = Header::from_str(
            r#"from :    "A. G. Bell"   <sip:agb@bell-telephone.com> ;   tag  = a48s"#,
        );
        if let Header::From(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"from :    "A. G. Bell"   <sip:agb@bell-telephone.com> ;   tag  = a48s"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"From: "A. G. Bell" <sip:agb@bell-telephone.com>;tag=a48s"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"f: "A. G. Bell" <sip:agb@bell-telephone.com>;tag=a48s"#
            );
        }
    }
}
