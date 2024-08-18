//! SIP version
//!
//! This module contains a definition of the SIP `Version` type. You should
//! not directly use the type from this module but rather the
//! `imersio_sip::Version` type.
//!
//! The `Version` type contains constants that represent the various versions
//! of the SIP protocol (only one version `SIP/2.0` for now).
//!
//! # Example
//!
//! ```
//! use imersio_sip::Version;
//! println!("{}", Version::Sip2);
//! ```

use crate::SipError;
use nom::error::convert_error;

/// Represents a version of the SIP specification.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Version {
    /// Version SIP/2.0.
    #[default]
    Sip2,
}

impl Version {
    /// Return a &str representation of the SIP method.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Version::Sip2 => "SIP/2.0",
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for Version {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for Version {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl PartialEq<Version> for &str {
    #[inline]
    fn eq(&self, other: &Version) -> bool {
        *self == other.as_ref()
    }
}

impl TryFrom<&str> for Version {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::sip_version(value) {
            Ok((rest, version)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(version)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidVersion(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidVersion(format!(
                "Incomplete version `{}`",
                value
            ))),
        }
    }
}

pub(crate) mod parser {
    use crate::{parser::ParserResult, Version};
    use nom::{bytes::complete::tag, combinator::value, error::context};

    pub(crate) fn sip_version(input: &str) -> ParserResult<&str, Version> {
        context("sip_version", value(Version::Sip2, tag("SIP/2.0")))(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::assert_err;

    #[test]
    fn test_version_eq() {
        assert_eq!(Version::Sip2.to_string(), "SIP/2.0");
        assert_eq!(Version::Sip2, "SIP/2.0");
        assert_eq!("SIP/2.0", Version::Sip2);
    }

    #[test]
    fn test_valid_version() {
        assert_eq!(Version::try_from("SIP/2.0").unwrap(), Version::Sip2);
    }

    #[test]
    fn test_invalid_version_empty() {
        assert_err!(Version::try_from(""));
    }

    #[test]
    fn test_invalid_version_unhandled_version() {
        assert_err!(Version::try_from("SIP/1.0"));
    }

    #[test]
    fn test_invalid_version_wrong_format() {
        assert_err!(Version::try_from("crappy-version"));
    }

    #[test]
    fn test_valid_version_but_with_remaining_data() {
        assert!(Version::try_from("SIP/2.0 anything")
            .is_err_and(|e| e == SipError::RemainingUnparsedData(" anything".to_string())));
    }
}
